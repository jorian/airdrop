/*

Airdrop
-------

- from snapshot
- from list of addresses
- define from-address
- define percentage / fixed amount to airdrop
- define which coin to airdrop (KMD or assetchain)

- z-support?
*/

use komodo_rpc_client::Chain;
use crate::snapshot::Snapshot;
use crate::error::AirdropError;
use crate::snapshot::Address;


// holds inputs to an airdrop transaction
// and holds outputs to that transaction, i.e., the participants in the airdrop
// in case of a non-multisig, it should be able to sign and broadcast (sign with wallet)
// in case of a multisig, it should be able to print the required signrawtransaction (done manually)
//   this string could (should) eventually have a wrapper for easy sharing and signing
pub struct Airdrop {
    // todo inputs?
    sourceaddress: SourceAddress,
    addresses: Vec<AddressPayout>,
    // todo Option<P2SH_Inputs>?
    multisig: bool,
}

impl Airdrop {
    pub fn builder() -> AirdropBuilder {

        Default::default()
    }

    pub fn signing_string() {
        // should return a string to sign
        // multisig should include P2SH inputs.
    }

    fn calculate()
}

pub struct AirdropBuilder {
    sourceaddress: Option<SourceAddress>,
    chain: Chain,
    payoutratio: f64,
    interest: bool,
    multisig: bool,
    snapshot: Option<Snapshot>,
}

// todo use a file with addresses as input, where file is able to be read by serde
// todo how to throw errors in a builder pattern?
impl AirdropBuilder {
    pub fn using_chain(&mut self, chain: Chain) -> &mut Self {
        self.chain = chain;

        self
    }

    pub fn using_snapshot(&mut self, snapshot: Snapshot) -> &mut Self {
        self.snapshot = Some(snapshot);

        self
    }

    pub fn source_address(&mut self, source: &str) -> &mut Self {
        self.sourceaddress = Some(SourceAddress(source.to_owned()));
        // recognize address: P2SH or P2PKH.
        self
    }

    pub fn payout_ratio(&mut self, ratio: f64) -> &mut Self {


        self
    }

    pub fn payout_amount(&mut self, satoshis: u64) -> &mut Self {
        // todo check balance for address and throw error when out of bounds

        self
    }

    pub fn include_interest(&mut self, include: bool) -> &mut Self {
        // todo throw error when not KMD

        self
    }

    pub fn configure(&self) -> Result<Airdrop, AirdropError> {
        let from_chain = self.chain;

        // an airdrop doesn't work without a snapshot, so chain is always set.
        let mut snapshot = self.snapshot.clone().unwrap();
//        let to_chain = snapshot.chain;

        let sourceaddress = self.sourceaddress.clone().unwrap();

        let addresses = snapshot.addresses;


        // todo Vec of snapshot addresses need to be converted, where we need to
        // properly divide funds over the snapshot addresses.


        Ok(Airdrop {
            sourceaddress: sourceaddress,
            addresses: vec![],
            multisig: false
        })
    }
}

impl Default for AirdropBuilder {
    fn default() -> Self {
        AirdropBuilder {
            sourceaddress: None,
            chain: Chain::KMD,
            payoutratio: 1.0,
            interest: false,
            multisig: false,
            snapshot: None,
        }
    }
}

pub struct AddressPayout {
    pub addr: String,
    pub amount: f64
}

#[derive(Debug, Clone)]
pub struct SourceAddress(String);

impl SourceAddress {
    pub fn is_valid(&self) -> bool {
        self.0.len() == 34
    }
}