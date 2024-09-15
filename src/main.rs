use std::{
    fs::File,
    io::{self, BufWriter, IsTerminal, Write},
    path::PathBuf,
};

use clap::Parser;
use solana_copy_trade_detect::{Args, RepeatingWallet};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // dotenvy::from_filename(".env.test").ok();

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
                println!("{}", serde_json::to_string(&repeating_wallets).unwrap());
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            eprintln!("Reach out to @Lezend on Discord for support");
        }
    }
}

/// Writes the repeating wallets and their transactions to a file.
///
/// This function creates a new file at the specified path and writes the details
/// of each repeating wallet, including the number of repeating transactions and their signatures.
///
/// # Arguments
///
/// * `repeating_wallets` - A vector of repeating wallets with their transactions.
/// * `file_path` - The path to the output file.
///
/// # Errors
///
/// This function will return an error if the file cannot be created or written to.
fn write_to_file(
    repeating_wallets: Vec<RepeatingWallet>,
    file_path: &PathBuf,
) -> Result<(), io::Error> {
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "Detected {} potential copy trading wallets",
        repeating_wallets.len()
    )?;
    writeln!(writer, "----------------------------------------")?;

    for item in repeating_wallets {
        writeln!(writer, "----------------------------------------")?;
        writeln!(writer, "Wallet: {}", item.wallet)?;
        writeln!(writer, "Number of copied swaps: {}", item.txs.len())?;
        writeln!(
            writer,
            "Swaps: {}",
            serde_json::to_string_pretty(&item.txs)?
        )?;
        writer.flush()?;
    }

    Ok(())
}
