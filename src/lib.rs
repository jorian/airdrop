#![feature(drain_filter)]
extern crate komodo_rpc_client;
#[macro_use]
extern crate derive_more;

mod airdrop;
mod snapshot;
mod error;

pub use komodo_rpc_client::Chain;
pub use crate::snapshot::{Snapshot, SnapshotBuilder};
pub use crate::airdrop::{Airdrop, AirdropBuilder};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
