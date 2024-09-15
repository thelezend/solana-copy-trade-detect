//! # Core Functionality
//!
//! This module contains the core functionality for the solana-copy-trade-detect application.
//!
//! The main function in this module is `run`, which orchestrates the fetching and processing
//! of fresh swap transactions and their previous buy transactions to detect copy trading wallets.

use std::{collections::HashMap, io::IsTerminal};

use cielo_rs_sdk::{
    api::feed::{Filters, TxType},
    models, CieloApi,
};
use shyft_rs_sdk::{
    models::parsed_transaction_details::{self, ParsedTransactionDetails},
    ShyftApi,
};

use crate::{print_if_terminal, PrevBuy};

/// Runs the main logic of the solana-copy-trade-detect application.
///
/// This function fetches fresh swap transactions and their previous buy transactions,
/// then filters and retains only wallets with more than one repeating previous buy.
///
/// # Arguments
///
/// * `args` - The command line arguments containing API keys, wallet address, and other parameters.
///
/// # Errors
///
/// This function will return an error if fetching fresh swaps or previous buys fails.
pub async fn run(args: &crate::Args) -> Result<Vec<(String, Vec<PrevBuy>)>, crate::Error> {
    let mut prev_wallets = HashMap::new();

    let fresh_swaps = fetch_fresh_swaps(args).await?;
    print_if_terminal!("Fetched {} fresh swaps", fresh_swaps.len());

    let shyft_api = ShyftApi::new(&args.shyft_api_key, None, None, None, None, None)?;

    for item in fresh_swaps.iter() {
        if let models::feed::Item::Swap(swap) = item {
            let prev_buys = fetch_prev_buys(args, &shyft_api, swap).await?;

            for buy in prev_buys.iter() {
                let raw_tx = buy.raw.as_ref().expect("raw tx data not found");
                let slot = raw_tx["slot"].as_u64().expect("slot not found");
                let block_diff = swap.block - slot;
                prev_wallets
                    .entry(buy.fee_payer.to_owned())
                    .or_insert_with(Vec::new)
                    .push(PrevBuy {
                        tx_hash: buy.signatures[0].to_owned(),
                        block_diff,
                    });

                // Sleep to avoid rate limit
                tokio::time::sleep(tokio::time::Duration::from_millis(args.delay_ms)).await;
            }
        }
    }

    // Retain only wallets with more than one repeating previous buy
    prev_wallets.retain(|_, buys| buys.len() > 1);

    // Sort the repeating wallets by the number of previous buys in descending order
    let mut repeating_wallets_vec: Vec<_> = prev_wallets.into_iter().collect();
    repeating_wallets_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    Ok(repeating_wallets_vec)
}

/// Fetches fresh swap transactions for a given wallet.
///
/// This function initializes a new Cielo API client and fetches the latest swap transactions
/// for the specified wallet based on the provided arguments.
///
/// # Arguments
///
/// * `args` - A reference to the arguments containing the API key, wallet address, and other parameters.
///
/// # Errors
///
/// This function will return an error if the Cielo API client could not be built or the request fails.
async fn fetch_fresh_swaps(
    args: &crate::Args,
) -> Result<Vec<models::feed::Item>, cielo_rs_sdk::Error> {
    let cielo_api = CieloApi::new(&args.cielo_api_key, None, None, None)?;
    cielo_api
        .get_feed(Filters {
            wallet: Some(args.wallet.to_string()),
            limit: Some(args.swap_num),
            chains: Some(vec!["solana".to_owned()]),
            tx_types: Some(vec![TxType::Swap]),
            new_trades: Some(true),
            ..Default::default()
        })
        .await
}

/// Fetches previous buy transactions for a given swap.
///
/// This function retrieves the transaction history for the specified token address
/// and filters the transactions to include only those that involve a swap where SOL is the input token.
///
/// # Arguments
///
/// * `args` - A reference to the arguments containing the API key, wallet address, and other parameters.
/// * `shyft_api` - A reference to the Shyft API client.
/// * `swap` - A reference to the swap transaction details.
///
/// # Errors
///
/// This function will return an error if the Shyft API request fails.
async fn fetch_prev_buys(
    args: &crate::Args,
    shyft_api: &ShyftApi,
    swap: &models::feed::Swap,
) -> Result<Vec<ParsedTransactionDetails>, shyft_rs_sdk::Error> {
    Ok(filter_buys(
        shyft_api
            .get_transaction_history(
                &swap.token1_address,
                Some(args.scan_tx_count),
                Some(&swap.tx_hash),
                None,
                Some(true),
                None,
            )
            .await?,
    ))
}

/// Filters transactions to include only those that involve a swap where SOL is the input token.
///
/// # Arguments
///
/// * `txs` - A vector of parsed transaction details.
///
/// # Returns
///
/// A vector of parsed transaction details that match the filter criteria.
fn filter_buys(txs: Vec<ParsedTransactionDetails>) -> Vec<ParsedTransactionDetails> {
    txs.into_iter()
        .filter(|tx| {
            tx.actions.iter().any(|a| {
                a.action_type == "SWAP"
                    && serde_json::from_value::<parsed_transaction_details::Swap>(a.info.clone())
                        .map_or(false, |info| info.tokens_swapped.token_in.symbol == "SOL")
            })
        })
        .collect()
}
