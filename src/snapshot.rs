/*

Snapshot
--------

- exclude addresses
- threshold

*/
use komodo_rpc_client::Client;
use komodo_rpc_client::Chain;
use std::default::Default;

pub struct Snapshot {
    addresses: Vec<String>,
    amount: f64,
}

pub struct Address {
    pub addr: String,
    pub amount: f64
}

#[derive(Debug)]
pub struct AddressPayout {
    pub addr: String,
    pub kmd_amount: i64
}

pub struct SnapshotBuilder {
    chain: Chain,
    threshold: f64,
    max_addresses: Option<u32>,
    excluded_addresses: Option<Vec<String>>
}

impl Snapshot {
}

impl SnapshotBuilder {
    fn new(chain: Chain) -> SnapshotBuilder {
        SnapshotBuilder {
            chain,
            threshold: Default::default(),
            max_addresses: Default::default(),
            excluded_addresses: Default::default()
        }
    }

    fn using_threshold(&mut self, threshold: f64) -> &mut Self {
        self.threshold;
        self
    }

    /// Include only the top `max` addresses in the snapshot. A max of 10 gives a snapshot of the
    /// top 10 addresses, based on their balance on their chain.
    fn max_addresses(&mut self, max: u32) -> &mut Self {
        self.max_addresses = Some(max);
        self
    }

    fn exclude_addresses(&mut self, addresses: Vec<String>) -> &mut Self {
        self.excluded_addresses = Some(addresses);
        self
    }

    fn make(&mut self) -> Snapshot {
        // a lot of code to do a snapshot, using komodod
        let client = match self.chain {
            Chain::KMD => komodo_rpc_client::Client::new_komodo_client(),
            _ => komodo_rpc_client::Client::new_assetchain_client(&self.chain)
        };

        // todo error handling
        let client = client.unwrap();

        // todo handle any error, after adding error handling
        let snapshot = match self.max_addresses {
            Some(max) => client.get_snapshot_max(max),
            None => client.get_snapshot()
        }.unwrap().unwrap();




        Snapshot {
            addresses: vec!["ab34234".to_string()],
            amount: 0.0
        }
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

