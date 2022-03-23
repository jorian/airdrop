//! # Komodo Airdrop
//!
//! (Komodo) Airdrop is a tool that allows anyone to perform an airdrop within the Komodo Platform.
//!
//! It builds on the komodo-rpc-client and offers an extension to the current Airdrop and Snapshot
//! functionality of the Komodo daemon.
//!
//! ## Snapshot
//!
//! Snapshot makes it easy to specify a Komodo (asset)chain and perform a snapshot on that chain.
//! It offers some extra properties, such as
//! - excluding specific addresses from the snapshot
//! - specifying a threshold to exclude addresses with low balance
//!
//!
//! ## Airdrop
//!
//! The airdrop functionality allows to specify a Komodo (asset)chain and do an airdrop on that chain using a Snapshot.
//! ### Why is this a useful tool?
//!
//! The Komodo platform allows anyone to create a blockchain. Every new blockchain is a runtime fork of the Komodo
//! blockchain, which means that these new blockchains inherit almost all functionalities from Komodo, including
//! address formats.
//! This allows the addresses in a snapshot to be used as an input on another blockchain within the platform, since
//! the private key to these addresses work throughout the Komodo platform and always lead to the same address.
//!
//! ### Notes
//! An airdrop (for now) only takes a Snapshot as input. In the future, a JSON file as input will be supported.
//!
//! ### Features
//! - Define a fund address that holds the coins to airdrop
//! - Airdrop a percentage of the fund address's funds, by applying a ratio between 0.0 and 1.0
//! - Airdrop a specific amount from the fund address
//! - Whether to include interest in the airdrop (KMD only)
//!
//! Assumes there are two blockchains running and synced:
//!  - the one where the snapshot is taken (usually an assetchain)
//!  - the one from where the funds are airdropped (mostly KMD)

#![feature(drain_filter)]
extern crate komodo_rpc;
#[macro_use]
extern crate derive_more;
extern crate komodo_snapshot;

mod airdrop;
// mod snapshot;
mod error;

// pub use komodo_rpc_client::Chain;
// pub use crate::snapshot::{Snapshot, SnapshotBuilder};
pub use crate::airdrop::{Airdrop, AirdropBuilder};
