use std::{
    fs::File,
    io::{self, BufWriter, IsTerminal, Write},
    path::PathBuf,
};

use clap::Parser;
use solana_copy_trade_detect::{Args, PrevBuy};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    dotenvy::from_filename(".env.test").ok();

    let args = Args::parse();
    let result = solana_copy_trade_detect::run(&args).await;

    match result {
        Ok(repeating_wallets) => {
            if io::stdout().is_terminal() {
                let file_path = args
                    .output_file
                    .unwrap_or(PathBuf::from(format!("{}.txt", args.wallet)));
                write_to_file(repeating_wallets, &file_path).expect("Failed to write to file");
                println!("Output written to {}", file_path.display());
            } else {
                println!("{:?}", repeating_wallets);
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            eprintln!("Reach out to @Lezend on Discord for support");
        }
    }
}

/// Writes the repeating wallets and their previous buy transactions to a file.
///
/// This function creates a new file at the specified path and writes the wallet addresses,
/// the number of transactions, and the signatures of the previous buy transactions.
///
/// # Arguments
///
/// * `repeating_wallets` - A vector of tuples containing wallet addresses and their previous buy transactions.
/// * `file_path` - The path to the file where the data will be written.
///
/// # Errors
///
/// This function will return an error if the file could not be created or written to.
fn write_to_file(
    repeating_wallets: Vec<(String, Vec<PrevBuy>)>,
    file_path: &PathBuf,
) -> Result<(), io::Error> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    for (wallet, prev_buys) in repeating_wallets {
        writeln!(writer, "Wallet: {}", wallet)?;
        writeln!(
            writer,
            "Number of repeating transactions: {}",
            prev_buys.len()
        )?;
        writeln!(writer, "Signatures: {:#?}", prev_buys)?;
        writer.flush()?;
    }

    Ok(())
}
