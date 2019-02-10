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


// holds inputs to an airdrop transaction
// and holds outputs to that transaction, i.e., the participants in the airdrop
// in case of a non-multisig, it should be able to sign and broadcast (sign with wallet)
// in case of a multisig, it should be able to print the required signrawtransaction (done manually)
//   this string could (should) eventually have a wrapper for easy sharing and signing
pub struct Airdrop {
    // todo inputs?
    payout_addresses: Vec<AddressPayout>
    // todo Option<P2SH_Inputs>?
}

impl Airdrop {
    pub fn builder() -> AirdropBuilder {

        Default::default()
    }

    pub fn signing_string() {
        // should return a string to sign
        // multisig should include P2SH inputs.
    }
}

pub struct AirdropBuilder {
    chain: Chain,
    payoutratio: f64,
    interest: bool,
}

// todo use a file with addresses as input, where file is able to be read by serde
impl AirdropBuilder {
    pub fn with_chain(&mut self, chain: Chain) -> &mut Self {


        self
    }

    pub fn using_snapshot(&mut self, snapshot: Snapshot) -> &mut Self {


        self
    }

    pub fn source_address(&mut self, source: &str) -> &mut Self {

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

    pub fn configure() {
        // todo should return Airdrop
    }
}

impl Default for AirdropBuilder {
    fn default() -> Self {
        AirdropBuilder {
            chain: Chain::KMD,
            payoutratio: 1.0,
            interest: false,
        }
    }
}

pub struct AddressPayout {
    pub addr: String,
    pub amount: f64
}