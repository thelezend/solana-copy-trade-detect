//! # Macros
//!
//! This module defines macros used in the solana-copy-trade-detect application.

/// Prints to the standard output if the output is a terminal.
///
/// This macro checks if the standard output is a terminal and prints the
/// formatted string if it is. It uses the `println!` macro internally.
#[macro_export]
macro_rules! print_if_terminal {
    ($($arg:tt)*) => {
        if std::io::stdout().is_terminal() {
            println!($($arg)*);
        }
    };
}

/// Creates and configures a new spinner with a custom message.
///
/// This macro initializes a new spinner using the `indicatif` crate, sets its style,
/// enables a steady tick, and assigns a custom message to it.
///
/// # Arguments
///
/// * `$msg` - The message to display with the spinner.
#[macro_export]
macro_rules! get_spinner {
    ($msg:expr) => {{
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
        spinner.enable_steady_tick(Duration::from_millis(120));
        spinner.set_message($msg);
        spinner
    }};
}
