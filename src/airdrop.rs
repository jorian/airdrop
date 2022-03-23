use crate::error::AirdropError;
use crate::error::ErrorKind;
use crate::snapshot::Snapshot;
use komodo_rpc::komodo_rpc_json::komodo::util::address::Address;
use komodo_rpc::komodo_rpc_json::AddressList;
use komodo_rpc::komodo_rpc_json::AddressUtxos;
use komodo_rpc::komodo_rpc_json::P2SHInputSetBuilder;
use komodo_rpc::RpcApi;
use serde_json;

// does not hold any inputs to a transaction: especially in the case of KMD, interest needs to be calculated right before
// an airdrop takes place
pub struct Airdrop {
    snapshot: Snapshot,
    fund_address: FundAddress,
    payout: Payout, //
    dest_addresses: Option<Vec<DestAddress>>,
}

impl Airdrop {
    pub fn builder<'a>() -> AirdropBuilder<'a> {
        Default::default()
    }

    /// Does a `createrawtransaction` RPC to the running daemon to create a raw transaction and uses the resulting hex to
    /// print the string that is needed for the `signrawtransaction` komodod RPC
    ///
    /// Because of multisig, signing is done manually
    pub fn signing_string(&self, redeem_script: Option<String>) -> Result<String, AirdropError> {
        let utxoset = self.get_current_utxoset()?;
        let inputs = komodo_rpc_client::arguments::CreateRawTransactionInputs::from(&utxoset);

        let mut outputs = komodo_rpc_client::arguments::CreateRawTransactionOutputs::new();
        for payout_addresses in &self.dest_addresses.clone().unwrap() {
            outputs.add(
                &Address::from(&payout_addresses.address.clone()).unwrap(),
                payout_addresses.amount as f64 / 100_000_000.0,
            );
        }

        let mut joined = String::new();

        if self.fund_address.multisig == true {
            if let Some(redeem_script) = redeem_script {
                let p2sh_input_set = P2SHInputSetBuilder::from(&utxoset)
                    .set_redeem_script(redeem_script)
                    .build()?;

                let p2sh_str = serde_json::to_string(&p2sh_input_set.0)?;
                joined = format!("\"{}\" \"[\"<WIF HERE>\"]\"", p2sh_str);
            }
        }

        let fund_client = match &self.fund_address.dest_chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.fund_address.dest_chain),
        }?;

        let mut crawtx = fund_client.create_raw_transaction(inputs, outputs)?;
        crawtx.set_locktime();

        joined = format!("{} {}", crawtx.0, joined);

        Ok(joined)
    }

    /// Takes the `fund_address` balance (and interest, if set) and calculates what each address in the
    /// snapshot will get, based on their `address balance` to `snapshot total balance` ratio
    ///
    /// Sets `dest_addresses`
    pub fn calculate(&mut self) -> Result<(), AirdropError> {
        // get the utxos for the fund_address
        let utxoset = self.get_current_utxoset()?;

        // get total balance of all utxos
        let mut balance = utxoset.0.iter().fold(0, |acc, utxo| acc + utxo.satoshis);

        // if chain is KMD, interest is needed (to airdrop or for change):
        let mut interest = 0;
        if self.fund_address.dest_chain == Chain::KMD {
            let client = komodo_rpc_client::Client::new_komodo_client()?;

            for utxo in utxoset.0 {
                let verbose_tx = client.get_raw_transaction_verbose(
                    komodo_rpc_client::TransactionId::from_hex(&utxo.txid).unwrap(),
                )?;

                interest += (verbose_tx
                    .vout
                    .get(utxo.output_index as usize)
                    .unwrap()
                    .interest
                    .unwrap()
                    * 100_000_000.0) as u64
            }
        }

        let snapshot_addresses = self.snapshot.addresses.clone();
        let denominator = snapshot_addresses
            .iter()
            .fold(0, |acc, x| acc + ((x.amount * 100_000_000.0) as u64));

        // this long match statement is needed for several scenarios:
        // - apply ratio or use amount,
        // - include interest
        match (&self.payout, self.fund_address.include_interest) {
            (Payout::Ratio(ratio), false) => {
                // apply ratio to balance
                // send remaining balance + interest back as change

                let airdrop_amt = (balance as f64 * ratio) as u64;
                let change = balance - airdrop_amt;

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((airdrop_amt as f64) * (addr.amount * 100_000_000.0)
                            / denominator as f64) as u64,
                    });
                }

                dest_addresses.push(DestAddress {
                    address: self.fund_address.address.clone(),
                    amount: (change + interest),
                });

                self.dest_addresses = Some(dest_addresses);
            }
            (Payout::Ratio(ratio), true) => {
                // add interest to balance
                // apply ratio to (balance + interest)
                // send back change

                balance = balance + interest;

                let airdrop_amt = (balance as f64 * ratio) as u64;
                let change = balance - airdrop_amt;

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((airdrop_amt as f64) * (addr.amount * 100_000_000.0)
                            / denominator as f64) as u64,
                    });
                }

                if change > 0 {
                    dest_addresses.push(DestAddress {
                        address: self.fund_address.address.clone(),
                        amount: change,
                    });
                }

                self.dest_addresses = Some(dest_addresses);
            }
            (Payout::Amount(amount), false) => {
                // airdrop payout_amount
                // send back remaining balance + interest

                let airdrop_amt = *amount;
                let change = balance - airdrop_amt;

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((airdrop_amt as f64) * (addr.amount * 100_000_000.0)
                            / denominator as f64) as u64,
                    });
                }

                dest_addresses.push(DestAddress {
                    address: self.fund_address.address.clone(),
                    amount: (change + interest),
                });

                self.dest_addresses = Some(dest_addresses);
            }
            (Payout::Amount(amount), true) => {
                // airdrop payout_amount + interest
                // send back remaining balance

                let airdrop_amt = *amount;
                let change = balance - airdrop_amt;
                let airdrop_amt = airdrop_amt + interest;

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((airdrop_amt as f64) * (addr.amount * 100_000_000.0)
                            / denominator as f64) as u64,
                    });
                }

                dest_addresses.push(DestAddress {
                    address: self.fund_address.address.clone(),
                    amount: change,
                });

                self.dest_addresses = Some(dest_addresses);
            }
            _ => panic!("ratio or amount not valid!"),
        }

        Ok(())
    }

    fn get_current_utxoset(&self) -> Result<AddressUtxos, AirdropError> {
        let client = match &self.fund_address.dest_chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.fund_address.dest_chain),
        }?;

        let mut address_list = AddressList::new();
        address_list.add(&self.fund_address.address);
        let utxo_set = client.get_address_utxos(&address_list)?;

        Ok(utxo_set)
    }
}

pub struct AirdropBuilder<'a> {
    chain: Chain,
    snapshot: Option<&'a Snapshot>,
    address: String,
    multisig: bool,
    interest: bool,
    ratio: Option<f64>,
    amount: Option<u64>,
}

impl<'a> AirdropBuilder<'a> {
    /// Specifies the blockchain to perform an airdrop on.
    pub fn using_chain(&mut self, chain: Chain) -> &mut Self {
        self.chain = chain;

        self
    }

    /// Sets the snapshot to be used in the airdrop.
    pub fn using_snapshot(&mut self, snapshot: &'a Snapshot) -> &mut Self {
        self.snapshot = Some(snapshot);

        self
    }

    /// Specifies the address that holds the funds to airdrop.
    pub fn fund_address(&mut self, source: &str) -> &mut Self {
        if source.len() != 34 {
            panic!("Source address length must be 34 chars.")
        }

        self.address = source.to_owned();

        match source.chars().next() {
            Some(letter) if letter == 'b' => self.multisig = true,
            _ => self.multisig = false,
        }

        self
    }

    /// Apply a ratio to the balance of the [fund_address](Airdrop::fund_address) to use in the airdrop
    /// calculation. Setting a ratio of 0.8 airdrops 80% of the funds in the [fund_address](Airdrop::fund_address).
    ///
    /// The ratio also applies to interest, if set. Using the same ratio of 0.8, 80% of the interest is airdropped,
    /// the remaining interest is included in the change, back to the [fund_address](Airdrop::fund_address).
    ///
    /// Panics if a zero or negative ratio or a ratio more than 1.0 is supplied.
    /// Panics if used together with [payout_amount](AirdropBuilder::payout_amount)
    pub fn payout_ratio(&mut self, ratio: f64) -> &mut Self {
        if ratio > 0.0 && ratio <= 1.0 {
            self.ratio = Some(ratio);
        } else {
            panic!("Ratio must be a float in range from 0.0 up to and including 1.0.");
        }

        self
    }

    /// Specify a fixed amount to use as input for the airdrop. If [interest](AirdropBuilder::include_interest) is included, 100% of the
    /// interest will be included in the funds to airdrop. Any change does not include interest.
    ///
    /// Panics if used together with [payout_ratio](AirdropBuilder::payout_ratio)
    pub fn payout_amount(&mut self, amount: f64) -> &mut Self {
        self.amount = Some((amount * 100_000_000.0) as u64);

        self
    }

    /// To properly check this, [using_chain()](AirdropBuilder::using_chain) must be specified before this function is called.
    /// Will be ignored if set on anything other than KMD.
    pub fn include_interest(&mut self, include: bool) -> &mut Self {
        match self.chain {
            Chain::KMD => self.interest = include,
            _ => self.interest = false,
        }

        self
    }

    pub fn build(&self) -> Result<Airdrop, AirdropError> {
        let snapshot = self.snapshot.unwrap();
        let ratio = self.ratio;

        let client = match &self.chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.chain),
        }?;

        let mut address_list = AddressList::new();
        address_list.add(&self.address);
        let addressbalance = client.get_address_balance(&address_list)?.balance;

        if let Some(amount) = self.amount {
            if addressbalance < amount {
                return Err(AirdropError::from(ErrorKind::BalanceInsufficient));
            }
        }

        let payout: Payout;
        match (self.ratio, self.amount) {
            (Some(ratio), None) => payout = Payout::Ratio(ratio),
            (None, Some(amount)) => payout = Payout::Amount(amount),
            (Some(_r), Some(_a)) => {
                return Err(AirdropError::from(ErrorKind::AmbiguousConfig(
                    String::from("Both ratio and payout_amount are defined"),
                )))
            }
            _ => {
                return Err(AirdropError::from(ErrorKind::AmbiguousConfig(
                    String::from("Neither ratio or payout_amount are defined"),
                )))
            }
        }

        let fund_address = FundAddress {
            address: self.address.clone(),
            dest_chain: self.chain.clone(),
            include_interest: self.interest,
            multisig: self.multisig,
        };

        Ok(Airdrop {
            fund_address,
            snapshot: snapshot.clone(),
            payout,
            dest_addresses: None,
        })
    }
}

impl<'a> Default for AirdropBuilder<'a> {
    fn default() -> Self {
        AirdropBuilder {
            chain: Chain::KMD,
            snapshot: None,
            address: String::new(),
            multisig: false,
            interest: false,
            ratio: None,
            amount: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DestAddress {
    pub address: String,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub struct FundAddress {
    address: String,
    dest_chain: Chain,
    include_interest: bool,
    multisig: bool,
}

impl Default for FundAddress {
    fn default() -> Self {
        FundAddress {
            address: String::new(),
            dest_chain: Chain::KMD,
            include_interest: false,
            multisig: false,
        }
    }
}

enum Payout {
    Ratio(f64),
    Amount(u64),
}
