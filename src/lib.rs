#![feature(drain_filter)]
extern crate komodo_rpc_client;

mod airdrop;
mod snapshot;

pub use snapshot::*;
pub use komodo_rpc_client::Chain;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
