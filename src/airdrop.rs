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
    ratio: f64,
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
        let fund_client = match &self.fund_address.dest_chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.fund_address.dest_chain)
        }?;

        let mut address_list = komodo_rpc_client::AddressList::new();
        address_list.add(&self.fund_address.address);

        let utxoset = fund_client.get_address_utxos(&address_list)?
            .unwrap();

        let balance = utxoset.0.iter()
            .fold(0, |acc, utxo| acc + utxo.satoshis);

        // if chain is KMD, interest is needed:
        let mut interest = 0;
        match self.fund_address.dest_chain {
            Chain::KMD => {
                interest = utxoset.0.iter()
                    .fold(0, |acc, utxo| {
                        (fund_client.get_raw_transaction_verbose(
                            komodo_rpc_client::TransactionId::from_hex(&utxo.txid).unwrap())
                            .unwrap()
                            .unwrap()
                            .vout.get(utxo.output_index as usize).unwrap().interest * 100_000_000.0) as u64
                    });
            },
            _ => ()
        }

            // add interest to balance
        if self.fund_address.include_interest {
            let balance = balance + interest;
        }

        // apply ratio:
        let balance = (balance as f64 * self.ratio) as u64;

        let snapshot_addresses = self.snapshot.addresses.clone();
        let denominator = snapshot_addresses.iter().fold(0, |acc, x| acc + ((x.amount * 100_000_000.0) as u64));

//        dbg!(denominator);

        let mut dest_addresses = vec![];
        for addr in snapshot_addresses {
//            dbg!(&a.amount);
            dest_addresses.push(DestAddress {
                address: addr.addr,
                amount: ((balance as f64) * (addr.amount * 100_000_000.0) / denominator as f64) as u64,
            });
        }

        let sum = dest_addresses.iter().fold(0, |acc, address| acc + address.amount);

//        dbg!(&payout_addresses);
        dbg!(sum);

        Ok(())
    }
}

pub struct AirdropBuilder {
    chain: Chain,
    snapshot: Option<Snapshot>,
    address: String,
    multisig: bool,
    interest: bool,
    ratio: f64
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

    pub fn fund_address(&mut self, source: &str) -> &mut Self {
        self.address = source.to_owned();

        match source.chars().next() {
            Some(letter) if letter == 'b' => self.multisig = true,
            _ => self.multisig = false
        }

        self
    }

    pub fn payout_ratio(&mut self, ratio: f64) -> &mut Self {
        match ratio {
            0.0..=1.0 => self.ratio = ratio,
            _ => panic!("Ratio is not between 0.0 and 1.0")
        }

        self
    }

    pub fn include_interest(&mut self, include: bool) -> &mut Self {
        self.interest = include;

        self
    }

    pub fn configure(&self) -> Result<Airdrop, AirdropError> {
        let snapshot = self.snapshot.clone().unwrap();
        let ratio = self.ratio;

        let fund_address = FundAddress {
            address: self.address.clone(),
            dest_chain: self.chain,
            include_interest: self.interest,
            multisig: self.multisig
        };


        Ok(Airdrop {
            fund_address,
            snapshot,
            ratio,
        })
    }
}

impl Default for AirdropBuilder {
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

#[derive(Debug)]
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

impl FundAddress {
    pub fn is_valid(&self) -> bool {
        self.address.len() == 34
    }
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