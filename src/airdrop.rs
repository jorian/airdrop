use komodo_rpc_client::KomodoRpcApi;
use komodo_rpc_client::Chain;
use crate::snapshot::Snapshot;
use crate::error::AirdropError;
use komodo_rpc_client::AddressUtxos;
use serde_json;
use komodo_rpc_client::AddressList;
use crate::error::ErrorKind;


// holds inputs to an airdrop transaction
// and holds outputs to that transaction, i.e., the participants in the airdrop
// in case of a non-multisig, it should be able to sign and broadcast (sign with wallet)
// in case of a multisig, it should be able to print the required signrawtransaction (done manually)
// this string could (should) eventually have a wrapper for easy sharing and signing
pub struct Airdrop {
    // todo CreateRawTransactionInputs?
    // todo Option<P2SH_Inputs>?

    snapshot: Snapshot,
    fund_address: FundAddress,
    ratio: Option<f64>,
    amount: Option<u64>,
    dest_addresses: Option<Vec<DestAddress>>,
}

impl Airdrop {
    pub fn builder<'a>() -> AirdropBuilder<'a> {

        Default::default()
    }

    /// Prints the string that is needed for the `signrawtransaction` komodod RPC
    pub fn signing_string(&self) -> Result<String, AirdropError> {
        // should return a string to sign
        // multisig should include P2SH inputs.

        let utxoset = self.get_current_utxoset()?;

        let mut inputs = komodo_rpc_client::arguments::CreateRawTransactionInputs::new();
        for utxo in utxoset.0 {
            inputs.add(&utxo.txid, utxo.output_index);
        }

        let mut outputs = komodo_rpc_client::arguments::CreateRawTransactionOutputs::new();
        for payout_addresses in &self.dest_addresses.clone().unwrap() {
            outputs.add(&payout_addresses.address.clone(), payout_addresses.amount as f64 / 100_000_000.0);
        }

        if self.fund_address.multisig == false {
            let inputs_str = serde_json::to_string(&inputs)?;
            let outputs_str = serde_json::to_string(&outputs)?;

            let joined = format!("{} {}", inputs_str, outputs_str);
            dbg!(joined);
        }

        Ok(String::new()) //todo return actual string
    }

    pub fn calculate(&mut self) -> Result<(), AirdropError> {
        // get a client
        let fund_client = match &self.fund_address.dest_chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.fund_address.dest_chain)
        }?;

        // get the utxos for the fund_address
        let mut address_list = komodo_rpc_client::AddressList::new();
        address_list.add(&self.fund_address.address);

        let utxoset = fund_client.get_address_utxos(&address_list)?;

        // get total balance of all utxos
        let mut balance = utxoset.0.iter()
            .fold(0, |acc, utxo| acc + utxo.satoshis);

        // if chain is KMD, interest is needed (to airdrop or for change):
        let mut interest = 0;
        match self.fund_address.dest_chain {
            Chain::KMD => {
                for utxo in utxoset.0 {
                    let verbose_tx = fund_client.get_raw_transaction_verbose(
                        komodo_rpc_client::TransactionId::from_hex(&utxo.txid).unwrap())?;

                    interest += (verbose_tx.vout.get(utxo.output_index as usize).unwrap().interest * 100_000_000.0) as u64
                }
            },
            _ => {}
        }

        let snapshot_addresses = self.snapshot.addresses.clone();
        let denominator = snapshot_addresses.iter().fold(0, |acc, x| acc + ((x.amount * 100_000_000.0) as u64));

        dbg!(&self.amount);

        match (self.ratio, self.amount, self.fund_address.include_interest) {
            (Some(ratio), None, false) if ratio == 1.0 => {
                // airdrop total balance
                // send interest back to fund_address

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((balance as f64) * (addr.amount * 100_000_000.0) / denominator as f64) as u64,
                    });
                }

                dest_addresses.push(DestAddress {
                    address: self.fund_address.address.clone(),
                    amount: interest
                });

                self.dest_addresses = Some(dest_addresses);
            },
            (Some(ratio), None, false) if ratio < 1.0 => {
                // apply ratio to balance
                // send remaining balance + interest back as change

                let airdrop_amt = (balance as f64 * ratio) as u64;
                let change = balance - airdrop_amt;

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((airdrop_amt as f64) * (addr.amount * 100_000_000.0) / denominator as f64) as u64,
                    });
                }

                dest_addresses.push(DestAddress {
                    address: self.fund_address.address.clone(),
                    amount: (change + interest)
                });

                self.dest_addresses = Some(dest_addresses);
            },
            (Some(ratio), None, true) if ratio == 1.0 => {
                // add interest to balance
                // airdrop balance
                // nothing to send back

                balance = balance + interest;

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((balance as f64) * (addr.amount * 100_000_000.0) / denominator as f64) as u64,
                    });
                };

                self.dest_addresses = Some(dest_addresses);
            },
            (Some(ratio), None, true) if ratio < 1.0 => {
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
                        amount: ((airdrop_amt as f64) * (addr.amount * 100_000_000.0) / denominator as f64) as u64,
                    });
                };

                dest_addresses.push(DestAddress {
                    address: self.fund_address.address.clone(),
                    amount: change
                });

                self.dest_addresses = Some(dest_addresses);
            },
            (None, Some(amount), false) => {
                // airdrop payout_amount
                // send back remaining balance + interest

                let airdrop_amt = amount;
                let change = balance - airdrop_amt;

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((airdrop_amt as f64) * (addr.amount * 100_000_000.0) / denominator as f64) as u64,
                    });
                }

                dest_addresses.push(DestAddress {
                    address: self.fund_address.address.clone(),
                    amount: (change + interest)
                });

                self.dest_addresses = Some(dest_addresses);
            },
            (None, Some(amount), true) => {
                // airdrop payout_amount + interest
                // send back remaining balance

                dbg!(balance);

                let airdrop_amt = amount;
                let change = balance - airdrop_amt;
                let airdrop_amt = airdrop_amt + interest;

                let mut dest_addresses = vec![];
                for addr in snapshot_addresses {
                    dest_addresses.push(DestAddress {
                        address: addr.addr,
                        amount: ((airdrop_amt as f64) * (addr.amount * 100_000_000.0) / denominator as f64) as u64,
                    });
                }

                dest_addresses.push(DestAddress {
                    address: self.fund_address.address.clone(),
                    amount: change
                });

                self.dest_addresses = Some(dest_addresses);
            },
            _ => panic!("both ratio and amount not valid!")
        }

        Ok(())
    }

    fn get_current_utxoset(&self) -> Result<AddressUtxos, AirdropError> {
        let client = match &self.fund_address.dest_chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.fund_address.dest_chain)
        }?;


        let mut address_list = komodo_rpc_client::AddressList::new();
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
    amount: Option<u64>
}

// todo use a file with addresses as input, where file is able to be read by serde
// todo how to throw errors in a builder pattern?
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
            _ => self.multisig = false
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

    /// To properly check this, [using_chain()](AirdropBuilder::using_chain) must come before this function call.
    /// Will be ignored if set on anything other than KMD.
    pub fn include_interest(&mut self, include: bool) -> &mut Self {
        match self.chain {
            Chain::KMD  => self.interest = include,
            _           => self.interest = false,
        }

        self
    }

    pub fn build(&self) -> Result<Airdrop, AirdropError> {
        let snapshot = self.snapshot.unwrap();
        let ratio = self.ratio;

        let client = match &self.chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.chain)
        }?;

        let mut address_list = AddressList::new();
        address_list.add(&self.address);
        let addressbalance = client.get_address_balance(&address_list)?.balance;

        if let Some(amount) = self.amount {
            if addressbalance < amount {
                return Err(AirdropError::from(ErrorKind::BalanceInsufficient))
            }
        }

        match (self.ratio, self.amount) {
            (Some(_r), Some(_a)) =>
                return Err(AirdropError::from(ErrorKind::AmbiguousConfig(String::from("Both ratio and payout_amount are defined")))),
            _ => { }
        }

        let fund_address = FundAddress {
            address: self.address.clone(),
            dest_chain: self.chain,
            include_interest: self.interest,
            multisig: self.multisig
        };

        Ok(Airdrop {
            fund_address,
            snapshot: snapshot.clone(),
            ratio,
            amount: self.amount,
            dest_addresses: None
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
            amount: None
        }
    }
}

#[derive(Debug, Clone)]
pub struct DestAddress {
    pub address: String,
    pub amount: u64
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