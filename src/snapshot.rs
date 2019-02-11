use komodo_rpc_client::Client;
use komodo_rpc_client::Chain;
use komodo_rpc_client::KomodoRpcApi;

use std::default::Default;

use crate::error::AirdropError;
use crate::error::ErrorKind;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub chain: Chain,
    pub addresses: Vec<Address>,
    pub amount: f64,
}

pub struct SnapshotBuilder {
    chain: Chain,
    threshold: f64,
    max_addresses: Option<u32>,
    excluded_addresses: Option<Vec<String>>
}

#[derive(Debug, Clone)]
pub struct Address {
    pub addr: String,
    pub amount: f64
}

impl Snapshot {
    pub fn builder() -> SnapshotBuilder {
        Default::default()
    }
}

impl SnapshotBuilder {
    pub fn on_chain(&mut self, chain: Chain) -> &mut Self {
        self.chain = chain;
        self
    }

    pub fn using_threshold(&mut self, threshold: f64) -> &mut Self {
        self.threshold = threshold;
        self
    }

    /// Include only the top `max` addresses in the snapshot. A max of 10 gives a snapshot of the
    /// top 10 addresses, based on their balance on their chain.
    pub fn max_addresses(&mut self, max: u32) -> &mut Self {
        self.max_addresses = Some(max);
        self
    }

    pub fn exclude_addresses(&mut self, addresses: Vec<String>) -> &mut Self {
        self.excluded_addresses = Some(addresses);
        self
    }

    pub fn make(&self) -> Result<Snapshot, AirdropError> {
        // a lot of code to do a snapshot, using komodod
        let client = match &self.chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client()?,
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.chain)?
        };

        // todo handle any error, after adding error handling
        let mut snapshot = match self.max_addresses {
            Some(max) => client.get_snapshot_max(max),
            None => client.get_snapshot()
        }?.unwrap();

        if snapshot.addresses.is_empty() {
            return Err(ErrorKind::EmptySnapshot.into())
        }

        if self.threshold > 0.0 {
            snapshot.addresses = snapshot.addresses
                .drain_filter(|saddress| saddress.amount > self.threshold)
                .collect::<Vec<_>>();
        }

        // first, remove any predefined excluded addresses from the snapshotted address vec
        // then, map each address and its corresponding amount to an Address struct.
        let addresses = snapshot.addresses
            .iter()
            .filter(|address| {
                let excluded_addresses = self.excluded_addresses.clone();
                match excluded_addresses  {
                    Some(vec) => !vec.contains(&address.addr),
                    None => return true
                }
            })
            .map(|address| Address { addr: address.addr.clone(), amount: address.amount })
            .collect::<Vec<_>>();

        Ok(Snapshot {
            chain: self.chain,
            addresses,
            amount: snapshot.total
        })
    }
}

impl Default for SnapshotBuilder {
    fn default() -> Self {
        SnapshotBuilder {
            chain: Chain::KMD,
            threshold: 0.0,
            max_addresses: None,
            excluded_addresses: None
        }
    }
}

