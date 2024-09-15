//! # Command Line Arguments
//!
//! This module defines the command line arguments for the solana-copy-trade-detect application.

use std::path::PathBuf;

use clap::Parser;
use solana_sdk::pubkey::Pubkey;

/// Command line arguments for the solana-copy-trade-detect application.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Cielo API key.
    #[arg(short, long, env = "CIELO_API_KEY")]
    pub cielo_api_key: String,
    /// Shyft API key.
    #[arg(short, long, env = "SHYFT_API_KEY")]
    pub shyft_api_key: String,
    /// The wallet to scan.
    #[arg(short, long)]
    pub wallet: Pubkey,
    /// Number of recent fresh swaps from the wallet to consider (max 100).
    #[arg(long, default_value = "15", value_parser = clap::value_parser!(u32).range(1..=100))]
    pub swap_num: u32,
    /// Number of transactions to scan for each swap to detect repeated wallets (max 100).
    #[arg(long, default_value = "100", value_parser = clap::value_parser!(u32).range(1..=100))]
    pub scan_tx_count: u32,
    /// Delay between Shyft API requests in milliseconds.
    #[arg(short, long, default_value = "500")]
    pub delay_ms: u64,
    /// Output file to write detected wallets. Default is wallet_address.txt.
    #[arg(short, long)]
    pub output_file: Option<PathBuf>,
}
