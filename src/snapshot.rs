/*

Snapshot
--------

- exclude addresses
- threshold

*/

pub struct Snapshot {
    pub start_time: u32,
    pub addresses: Vec<Address>,
    pub total: f64,
    pub average: f64,
    pub utxos: u32,
    pub total_addresses: u32,
    pub start_height: u32,
    pub ending_height: u32,
    pub end_time: u32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Address {
    pub addr: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub amount: f64
}

#[derive(Debug)]
pub struct AddressPayout {
    pub addr: String,
    pub kmd_amount: i64
}

pub struct SnapshotBuilder {
    chain: ::Chain,
    threshold: f64
}

impl Snapshot {
    // uses a client
    fn make()
}

impl SnapshotBuilder {
    fn new(chain: ::Chain) -> SnapshotBuilder {

    }

    fn using_threshold(&mut self, threshold: f64) -> self {

    }

    fn max_addresses(&mut self, max: u32) -> self {

    }
}

