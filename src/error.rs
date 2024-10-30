//! # Errors
//!
//! This module defines the different errors that can occur in the solana-copy-trade-detect application.
//!
//! The `Error` enum represents various error types that may be encountered during the execution of the application.

/// Represents the different errors that can occur in the solana-copy-trade-detect application.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error that occurs when fetching fresh swaps fails.
    #[error("Failed to fetch fresh swaps: {0}")]
    FetchFreshSwaps(#[from] cielo_rs_sdk::Error),

    /// Error that occurs when there is an issue with the Shyft API.
    #[error("Shyft API error: {0}")]
    ShyftApi(#[from] shyft_rs_sdk::Error),

    /// Error that occurs when there is an issue with the Solana RPC client.
    #[error("Solana RPC client error: {0}")]
    RpcClient(#[from] solana_client::client_error::ClientError),

    /// Error that occurs when fetching previous buy transactions fails.
    #[error("Failed to fetch previous buy transactions: {0}")]
    PrevBuysFetch(#[from] PrevBuysFetchError),
}

/// Represents the errors that can occur while fetching previous buy transactions.
#[derive(thiserror::Error, Debug)]
pub enum PrevBuysFetchError {
    /// Error that occurs when there is an issue with the Shyft API.
    #[error("Shyft API error: {0}")]
    ShyftApi(#[from] shyft_rs_sdk::Error),

    /// Error that occurs when there is an issue with the Solana RPC client.
    #[error("Solana RPC client error: {0}")]
    RpcClient(#[from] solana_client::client_error::ClientError),
}
