//! # Spinner Module
//!
//! Module that handles the progress indicator.

use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates the progress indicator.
///
/// ### Parameters
///
/// - `message`: A message to display with the progress indicator.
///
/// ### Returns
///
/// - `ProgressBar`: The configured progress bar.
pub fn setup_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(120));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[".", "..", "...", "....", "....."])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_owned());
    spinner
}

/// Suspends the spinner to print intermediate status messages.
///
/// ### Parameters
///
/// - `message`: The message to print.
/// - `success`: Whether the message is a success or failure message.
/// - `spinner`: Reference to the progress bar.
///
pub fn print_status(message: &str, success: bool, spinner: &ProgressBar) {
    if success {
        println!(
            "{}{}{} {}",
            "[".bold().white(),
            "✓".bold().green(),
            "]".bold().white(),
            message.green()
        );
    } else {
        eprintln!(
            "{}{}{} {}",
            "[".bold().white(),
            "!".bold().red(),
            "]".bold().white(),
            message.red()
        );
        spinner.finish();
    }
}
