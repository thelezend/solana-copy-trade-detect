use std::{
    fs::File,
    io::{self, BufWriter, IsTerminal, Write},
    path::PathBuf,
    time::Duration,
};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use solana_copy_trade_detect::{get_spinner, Args, RepeatingWallet};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Setup logging
    // Create a file layer with debug level filtering
    let file_appender = tracing_appender::rolling::daily("logs", ".log");
    let (file_writer, _file_writer_guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(file_writer)
        .with_filter(EnvFilter::new("solana_copy_trade_detect::core=debug"));

    tracing_subscriber::registry().with(file_layer).init();

    let args = Args::parse();
    let result = solana_copy_trade_detect::run(&args).await;

    match result {
        Ok(repeating_wallets) => {
            if io::stdout().is_terminal() {
                let file_path = args
                    .output_file
                    .unwrap_or(PathBuf::from(format!("{}.txt", args.wallet)));

                let spinner = get_spinner!(format!(
                    "{} {}Writing output to {}",
                    console::style("[3/3]").bold().dim(),
                    solana_copy_trade_detect::FILE,
                    file_path.display()
                ));

                write_to_file(repeating_wallets, &file_path).expect("Failed to write to file");
                spinner.finish();

                println!("\t\t{}Done!", solana_copy_trade_detect::CHECK);
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
        "Detected {} potential copied wallets",
        repeating_wallets.len()
    )?;

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
