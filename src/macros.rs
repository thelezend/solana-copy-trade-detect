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
