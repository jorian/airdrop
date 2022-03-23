// // use komodo_rpc_client::Chain;
// // use komodo_rpc_client::KomodoRpcApi;

// use komodo_rpc::Client; 
// use komodo_rpc::Auth;
// use std::default::Default;

// use crate::error::AirdropError;
// use crate::error::ErrorKind;

// /// Ultimately holds the details of a snapshot, performed on a Komodo (asset)chain.
// ///
// /// The snapshot can be used to perform an Airdrop.

// #[derive(Debug, Clone)]
// pub struct Snapshot {
//     pub chain: Chain,
//     pub addresses: Vec<Address>,
//     pub amount: f64,
// }

// pub struct SnapshotBuilder {
//     chain: Chain,
//     threshold: f64,
//     max_addresses: Option<u32>,
//     excluded_addresses: Option<Vec<String>>
// }

// #[derive(Debug, Clone)]
// pub struct Address {
//     pub addr: String,
//     pub amount: f64
// }

// impl Snapshot {
//     pub fn builder() -> SnapshotBuilder {
//         Default::default()
//     }
// }

// impl SnapshotBuilder {
//     /// Specify the Komodo (asset)chain to take a snapshot from.
//     pub fn on_chain(&mut self, chain: Chain) -> &mut Self {
//         self.chain = chain;
//         self
//     }

//     /// Set a threshold, such that all addresses contain at least the specified threshold.
//     pub fn using_threshold(&mut self, threshold: f64) -> &mut Self {
//         self.threshold = threshold;
//         self
//     }

//     /// Include only the top `max` addresses in the snapshot. A max of 10 gives a snapshot of the
//     /// top 10 addresses, based on their balance on their chain.
//     pub fn max_addresses(&mut self, max: u32) -> &mut Self {
//         self.max_addresses = Some(max);
//         self
//     }

//     /// Removes the addresses specified here from the Snapshot, if they exist in the Snapshot.
//     pub fn exclude_addresses(&mut self, addresses: Vec<String>) -> &mut Self {
//         self.excluded_addresses = Some(addresses);
//         self
//     }

//     /// Builds a Snapshot struct. Here is where the threshold is applied and excluded addresses are removed, if any.
//     pub fn build(&self) -> Result<Snapshot, AirdropError> {
//         // a lot of code to do a snapshot, using komodod
//         let client = Client::new(&self.chain, Auth::ConfigFile)?,
        
//         // todo handle any error, after adding error handling
//         let mut snapshot = match self.max_addresses {
//             Some(max) => client.get_snapshot(Some(String::from(max))),
//             None => client.get_snapshot(None)
//         }?;

//         if snapshot.addresses.is_empty() {
//             return Err(ErrorKind::EmptySnapshot.into())
//         }

//         if self.threshold > 0.0 {
//             snapshot.addresses = snapshot.addresses
//                 .drain_filter(|saddress| saddress.amount > self.threshold)
//                 .collect::<Vec<_>>();
//         }

//         // first, remove any predefined excluded addresses from the snapshotted address vec
//         // then, map each address and its corresponding amount to an Address struct.
//         let addresses = snapshot.addresses
//             .iter()
//             .filter(|address| {
//                 let excluded_addresses = self.excluded_addresses.clone();
//                 match excluded_addresses  {
//                     Some(vec) => !vec.contains(&address.addr),
//                     None => return true
//                 }
//             })
//             .map(|address| Address { addr: address.addr.clone(), amount: address.amount })
//             .collect::<Vec<_>>();

//         Ok(Snapshot {
//             chain: self.chain.clone(),
//             addresses,
//             amount: snapshot.total
//         })
//     }
// }

// impl Default for SnapshotBuilder {
//     fn default() -> Self {
//         SnapshotBuilder {
//             chain: Chain::KMD,
//             threshold: 0.0,
//             max_addresses: None,
//             excluded_addresses: None
//         }
//     }
// }

