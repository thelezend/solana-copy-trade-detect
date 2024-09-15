//! # Solana Copy Trade Detect
//!
//! This crate provides functionality to detect if a given Solana wallet is copy trading.

#![warn(
    missing_docs,
    rustdoc::unescaped_backticks,
    clippy::missing_errors_doc,
    clippy::missing_docs_in_private_items
)]

mod args;
mod core;
mod error;
mod macros;

pub use args::Args;
pub use core::run;
pub use error::Error;

/// Represents a repeating wallet with its transactions.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RepeatingWallet {
    /// The wallet address.
    pub wallet: String,
    /// The transactions.
    pub txs: Vec<PrevBuy>,
}

/// Represents a previous buy transaction with its hash and block difference.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PrevBuy {
    /// The transaction hash.
    pub tx_hash: String,
    /// The difference in blocks.
    pub block_diff: u64,
}
