//! # Core Functionality
//!
//! This module contains the core functionality for the solana-copy-trade-detect application.
//!
//! The main function in this module is `run`, which orchestrates the fetching and processing
//! of fresh swap transactions and their previous buy transactions to detect copy trading wallets.

use std::{collections::HashMap, str::FromStr, time::Duration};

use cielo_rs_sdk::{
    api::feed::{Filters, TxType},
    models, CieloApi,
};
use indicatif::{ProgressBar, ProgressStyle};
use shyft_rs_sdk::{
    models::parsed_transaction_details::{self, ParsedTransactionDetails},
    ShyftApi,
};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_client::GetConfirmedSignaturesForAddress2Config,
    rpc_config::RpcTransactionConfig,
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;

use crate::{error::PrevBuysFetchError, get_spinner, PrevBuy, RepeatingWallet};

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
pub async fn run(args: &crate::Args) -> Result<Vec<RepeatingWallet>, crate::Error> {
    let mut prev_wallets = HashMap::new();

    let spinner = get_spinner!(format!(
        "{} {}Fetching fresh swaps...",
        console::style("[1/3]").bold().dim(),
        crate::LIGHTNING,
    ));
    let fresh_swaps = fetch_fresh_swaps(args).await?;
    spinner.finish();

    tracing::info!("Fetched {} fresh swaps", fresh_swaps.len());

    if fresh_swaps.is_empty() {
        eprintln!(
            "\n{}",
            console::style("Error: Cielo API returned no swaps for the given wallet.")
                .red()
                .bold()
        );
        eprintln!("This may happen if the wallet is not on your watchlist or is a bot wallet that Cielo does not support. Please check the wallet page on Cielo.");
        eprintln!("https://app.cielo.finance/profile/{}\n", args.wallet);
        eprintln!("Exiting...");
        std::process::exit(1);
    }

    let progress_bar = ProgressBar::new(fresh_swaps.len() as u64);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} {msg} [{wide_bar:.green/magenta}] {percent}%",
        )
        .unwrap(),
    );
    progress_bar.enable_steady_tick(Duration::from_millis(120));
    progress_bar.set_message(format!(
        "{} {}Scanning transaction history...",
        console::style("[2/3]").bold().dim(),
        crate::SCAN,
    ));

    let shyft_api = ShyftApi::new(&args.shyft_api_key, None, None, None, None, None)?;

    // Use Shyft RPC if no RPC URL is provided
    let rpc_url = args
        .rpc_url
        .as_ref()
        .unwrap_or(&format!(
            "https://rpc.shyft.to/?api_key={}",
            args.shyft_api_key
        ))
        .to_owned();
    let rpc_client = RpcClient::new_with_timeout(rpc_url, Duration::from_secs(10));

    for item in fresh_swaps.iter() {
        if let models::feed::Item::Swap(swap) = item {
            let prev_buys =
                fetch_prev_buys(args, &rpc_client, &shyft_api, swap, args.delay_ms).await?;

            tracing::info!("Fetched {} previous buys", prev_buys.len());

            for buy in prev_buys.iter() {
                let block_diff = get_block_diff(&rpc_client, swap, buy, args.delay_ms).await?;
                prev_wallets
                    .entry(buy.fee_payer.to_owned())
                    .or_insert_with(Vec::new)
                    .push(PrevBuy {
                        tx_hash: buy.signatures[0].to_owned(),
                        block_diff,
                    });
            }
            // Sleep to avoid rate limit
            tokio::time::sleep(tokio::time::Duration::from_millis(args.delay_ms)).await;
        }
        progress_bar.inc(1);
    }

    progress_bar.finish();

    // Retain only wallets with more than one repeating previous buy
    prev_wallets.retain(|_, buys| buys.len() > 1);

    // Sort the repeating wallets by the number of previous buys in descending order
    let mut repeating_wallets_vec: Vec<_> = prev_wallets
        .into_iter()
        .map(|(wallet, buys)| RepeatingWallet { wallet, txs: buys })
        .collect();
    repeating_wallets_vec.sort_by(|a, b| b.txs.len().cmp(&a.txs.len()));

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
/// * `rpc_client` - A reference to the Solana RPC client.
/// * `shyft_api` - A reference to the Shyft API client.
/// * `swap` - A reference to the swap transaction details.
/// * `delay_ms` - The delay in milliseconds between requests.
///
/// # Errors
///
/// This function will return an error if the Shyft API request fails or if the transaction parsing fails.
async fn fetch_prev_buys(
    args: &crate::Args,
    rpc_client: &RpcClient,
    shyft_api: &ShyftApi,
    swap: &models::feed::Swap,
    delay_ms: u64,
) -> Result<Vec<ParsedTransactionDetails>, PrevBuysFetchError> {
    let successful_signatures =
        fetch_successful_signatures(rpc_client, swap, args.scan_tx_count as usize, delay_ms)
            .await?;
    tracing::info!(
        "Fetched {} successful signatures",
        successful_signatures.len()
    );

    if successful_signatures.is_empty() {
        tracing::warn!("No successful signatures found");
        return Ok(Vec::new());
    }

    let parsed_txs = shyft_api
        .get_transaction_parse_selected(
            &successful_signatures
                [..std::cmp::min(successful_signatures.len(), args.scan_tx_count as usize)],
            Some(true),
            None,
        )
        .await?;

    Ok(filter_buys(parsed_txs))
}

/// Fetches successful transaction signatures for a given swap.
///
/// This function retrieves the transaction signatures for the specified token address
/// and filters the signatures to include only those that are successful.
///
/// # Arguments
///
/// * `rpc_client` - A reference to the Solana RPC client.
/// * `swap` - A reference to the swap transaction details.
/// * `scan_tx_count` - The number of transaction signatures to scan.
/// * `delay_ms` - The delay in milliseconds between requests.
///
/// # Errors
///
/// This function will return an error if the Solana RPC request fails.
async fn fetch_successful_signatures(
    rpc_client: &RpcClient,
    swap: &models::feed::Swap,
    scan_tx_count: usize,
    delay_ms: u64,
) -> Result<Vec<String>, solana_client::client_error::ClientError> {
    let mut successful_signatures = Vec::new();

    let mut before_tx = Signature::from_str(&swap.tx_hash).unwrap();
    while successful_signatures.len() < scan_tx_count {
        let token1_address = if swap.token1_address == "native" {
            "So11111111111111111111111111111111111111112"
        } else {
            &swap.token1_address
        };
        let tx_signatures = rpc_client
            .get_signatures_for_address_with_config(
                &Pubkey::from_str(token1_address).unwrap(),
                GetConfirmedSignaturesForAddress2Config {
                    before: Some(before_tx),
                    until: None,
                    limit: None,
                    commitment: Some(CommitmentConfig::confirmed()),
                },
            )
            .await?;

        tracing::debug!("Fetched {} signatures", tx_signatures.len());

        if tx_signatures.is_empty() {
            break;
        }
        before_tx = Signature::from_str(&tx_signatures.last().unwrap().signature).unwrap();

        for signature in tx_signatures.iter() {
            if signature.err.is_none() {
                successful_signatures.push(signature.signature.to_string());
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    }

    Ok(successful_signatures)
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

/// Calculates the block difference between a fresh swap and a previous buy transaction.
///
/// This function fetches the block number of the previous buy transaction and compares it with the
/// block number of the fresh swap. If the block number of the fresh swap is not available, it fetches
/// it from the Solana RPC client.
///
/// # Arguments
///
/// * `rpc_client` - A reference to the Solana RPC client.
/// * `fresh_swap` - A reference to the fresh swap transaction details.
/// * `prev_buy` - A reference to the previous buy transaction details.
/// * `delay_ms` - The delay in milliseconds before fetching the fresh swap block number.
///
/// # Errors
///
/// This function will return an error if the Solana RPC request fails.
async fn get_block_diff(
    rpc_client: &RpcClient,
    fresh_swap: &models::feed::Swap,
    prev_buy: &ParsedTransactionDetails,
    delay_ms: u64,
) -> Result<u64, solana_client::client_error::ClientError> {
    let raw_tx = prev_buy.raw.as_ref().expect("raw tx data not found");
    let prev_buy_block = raw_tx["slot"].as_u64().expect("slot not found");

    // This is necessary because the Cielo API sometimes returns incorrect block data
    if let Some(block_diff) = fresh_swap.block.checked_sub(prev_buy_block) {
        Ok(block_diff)
    } else {
        let signature = Signature::from_str(&fresh_swap.tx_hash).unwrap();
        let fresh_swap_block = rpc_client
            .get_transaction_with_config(
                &signature,
                RpcTransactionConfig {
                    encoding: Some(UiTransactionEncoding::JsonParsed),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            )
            .await?
            .slot;
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        Ok(fresh_swap_block - prev_buy_block)
    }
}
