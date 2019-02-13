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
use crate::snapshot::Address;


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
    dest_chain: Chain,
    include_interest: bool,
    ratio: f64,
    multisig: bool,
}

impl Airdrop {
    pub fn builder() -> AirdropBuilder {

        Default::default()
    }

    pub fn signing_string(&self) {
        // should return a string to sign
        // multisig should include P2SH inputs.
    }

    pub fn calculate(&self) -> Result<(), AirdropError> {
        let source_client = match &self.from_chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.from_chain)
        }?;

        let mut source_address = komodo_rpc_client::AddressList::new();
        source_address.add(&self.source_address.0);

        let utxoset = source_client.get_address_utxos(&source_address)?
            .unwrap();

        let balance = match (self.include_interest, self.dest_chain) {
            (true, Chain::KMD) => {
                utxoset.0.iter()
                    .fold(0, |acc, utxo|
                        {
                            (source_client.get_raw_transaction_verbose(
                                komodo_rpc_client::TransactionId::from_hex(&utxo.txid).unwrap())?
                                .unwrap()
                                .vout.get(utxo.output_index as usize).unwrap().interest * 100_000_000.0) as u64
                        }
                        + acc + utxo.satoshis)
            }
            _ => utxoset.0.iter()
                .fold(0, |acc, utxo| acc + utxo.satoshis)
        };

        // apply ratio:
        let balance = (balance as f64 * self.ratio) as u64;

        let participants = self.snapshot.addresses.clone();
        let denominator = participants.iter().fold(0, |acc, x| acc + ((x.amount * 100_000_000.0) as u64));

//        dbg!(denominator);

        let mut payout_addresses = vec![];
        for a in participants {
//            dbg!(&a.amount);
            payout_addresses.push(DestAddress {
                address: a.addr,
                amount: ((balance as f64) * (a.amount * 100_000_000.0) / denominator as f64) as u64,
            });
        }

        let sum = payout_addresses.iter().fold(0, |acc, address| acc + address.sat_amount);

//        dbg!(&payout_addresses);
        dbg!(sum);

        Ok(())
    }
}

pub struct AirdropBuilder {
    snapshot: Option<Snapshot>,
    fund_address: Option<FundAddress>,
    chain: Chain,
    interest: bool,
    payoutratio: f64,
    multisig: bool,
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
        self.fund_address = Some(FundAddress(source.to_owned()));
        // recognize address: P2SH or P2PKH.
        self
    }

    pub fn payout_ratio(&mut self, ratio: f64) -> &mut Self {
        match ratio {
            0.0..=1.0 => self.payoutratio = ratio,
            _ => panic!("Ratio is not between 0.0 and 1.0")
        }


        self
    }

    pub fn payout_amount(&mut self, satoshis: u64) -> &mut Self {
        // todo check balance for address and throw error when out of bounds

        self
    }

    pub fn include_interest(&mut self, include: bool) -> &mut Self {
        self.interest = include;

        self
    }

    pub fn configure(&self) -> Result<Airdrop, AirdropError> {
        // an airdrop doesn't work without a snapshot, so chain is always set.
        let mut snapshot = self.snapshot.clone().unwrap();
//        let to_chain = snapshot.chain;

        let sourceaddress = self.fund_address.clone().unwrap();

        let ratio = self.payoutratio;


        Ok(Airdrop {
            dest_chain: self.chain,
            fund_address: sourceaddress,
            multisig: false,
            include_interest: self.interest,
            snapshot: snapshot,
            ratio: ratio,
        })
    }
}

impl Default for AirdropBuilder {
    fn default() -> Self {
        AirdropBuilder {
            fund_address: None,
            chain: Chain::KMD,
            payoutratio: 1.0,
            interest: false,
            multisig: false,
            snapshot: None,
        }
    }
}

#[derive(Debug)]
pub struct DestAddress {
    pub address: String,
    pub amount: u64
}

#[derive(Debug, Clone)]
pub struct FundAddress(String);

impl FundAddress {
    pub fn is_valid(&self) -> bool {
        self.0.len() == 34
    }
}