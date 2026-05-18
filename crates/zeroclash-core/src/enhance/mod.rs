//! Profile enhancement pipeline — processes clash configs through merge, sequence ops,
//! chain items, TUN configuration, and field ordering.
//!
//! This module is adapted from clash-verge-rev's `src-tauri/src/enhance/`.
//! Script (Boa JS) execution is deferred to a later phase.

mod chain;
mod field;
mod merge;
pub mod seq;
mod tun;

pub use chain::{ChainItem, ChainType};
pub use field::{use_keys, use_lowercase, use_sort};
pub use merge::use_merge;
pub use seq::{use_seq, SeqMap};
pub use tun::use_tun;
