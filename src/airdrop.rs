/*

Airdrop
-------

- from snapshot
- from list of addresses
- define from-address
- define percentage / fixed amount to airdrop
- define which coin to airdrop (KMD or assetchain)

- z-support?

- assumes there are two blockchains running and synced:
    - the one where the snapshot takes place (usually an assetchain)
    - the one from where the funds are airdropped (mostly KMD)
*/

use komodo_rpc_client::KomodoRpcApi;
use komodo_rpc_client::Chain;
use crate::snapshot::Snapshot;
use crate::error::AirdropError;
use komodo_rpc_client::AddressUtxos;
use serde_json;


// holds inputs to an airdrop transaction
// and holds outputs to that transaction, i.e., the participants in the airdrop
// in case of a non-multisig, it should be able to sign and broadcast (sign with wallet)
// in case of a multisig, it should be able to print the required signrawtransaction (done manually)
//   this string could (should) eventually have a wrapper for easy sharing and signing
pub struct Airdrop {
    // todo CreateRawTransactionInputs?
    // todo Option<P2SH_Inputs>?

    snapshot: Snapshot,
    fund_address: FundAddress,
    ratio: f64,
    dest_addresses: Option<Vec<DestAddress>>,
}

impl Airdrop {
    pub fn builder<'a>() -> AirdropBuilder<'a> {

        Default::default()
    }

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

        Ok(String::new())
    }

    pub fn calculate(&mut self) -> Result<(), AirdropError> {
        let fund_client = match &self.fund_address.dest_chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.fund_address.dest_chain)
        }?;

        let mut address_list = komodo_rpc_client::AddressList::new();
        address_list.add(&self.fund_address.address);

        let utxoset = fund_client.get_address_utxos(&address_list)?
            .unwrap();

        let mut balance = utxoset.0.iter()
            .fold(0, |acc, utxo| acc + utxo.satoshis);

        // if chain is KMD, interest is needed:
        let mut interest = 0;
        match self.fund_address.dest_chain {
            Chain::KMD => {
                for utxo in utxoset.0 {
                    let verbose_tx = fund_client.get_raw_transaction_verbose(
                        komodo_rpc_client::TransactionId::from_hex(&utxo.txid).unwrap())?.unwrap();

                    interest += (verbose_tx.vout.get(utxo.output_index as usize).unwrap().interest * 100_000_000.0) as u64
                }
            },
            _ => ()
        }

        // add interest to balance
        if self.fund_address.include_interest {
            balance = balance + interest;
        }

        // apply ratio:
        balance = (balance as f64 * self.ratio) as u64;

        let snapshot_addresses = self.snapshot.addresses.clone();
        let denominator = snapshot_addresses.iter().fold(0, |acc, x| acc + ((x.amount * 100_000_000.0) as u64));

        let mut dest_addresses = vec![];
        for addr in snapshot_addresses {
            dest_addresses.push(DestAddress {
                address: addr.addr,
                amount: ((balance as f64) * (addr.amount * 100_000_000.0) / denominator as f64) as u64,
            });
        }

        self.dest_addresses = Some(dest_addresses);

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

        Ok(utxo_set.unwrap())
    }
}

pub struct AirdropBuilder<'a> {
    chain: Chain,
    snapshot: Option<&'a Snapshot>,
    address: String,
    multisig: bool,
    interest: bool,
    ratio: f64
}

// todo use a file with addresses as input, where file is able to be read by serde
// todo how to throw errors in a builder pattern?
impl<'a> AirdropBuilder<'a> {
    pub fn using_chain(&mut self, chain: Chain) -> &mut Self {
        self.chain = chain;

        self
    }

    pub fn using_snapshot(&mut self, snapshot: &'a Snapshot) -> &mut Self {
        self.snapshot = Some(snapshot);

        self
    }

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

    pub fn payout_ratio(&mut self, ratio: f64) -> &mut Self {
        if ratio > 0.0 && ratio <= 1.0 {
            self.ratio = ratio;
        } else {
            panic!("Ratio must be a float in range from 0.0 up to and including 1.0.");
        }

        self
    }

    pub fn include_interest(&mut self, include: bool) -> &mut Self {
        self.interest = include;

        self
    }

    pub fn configure(&self) -> Result<Airdrop, AirdropError> {
        let snapshot = self.snapshot.unwrap();
        let ratio = self.ratio;

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
            ratio: 0.0
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