use clap::{ArgAction, Parser};
use leads::{prelude::*, spinner};
use std::path::PathBuf;

/// Command-line arguments for the LEADS application.
#[derive(Parser, Debug)]
#[clap(name = "LEADS", version = "0.0.1")]
struct Args {
    /// Path to the file to generate a report for.
    #[arg()]
    path: PathBuf,

    /// Indicates whether the file has headers. Defaults to True.
    #[arg(short = 'r', long, action(ArgAction::SetFalse))]
    headers: bool,

    /// Whether a progress spinner and status messages should be printed.
    #[arg(long, action(ArgAction::SetTrue))]
    verbose: bool,
}

fn main() -> LeadsResult<()> {
    let args = Args::parse();

    let spinner = if args.verbose {
        Some(spinner::setup_spinner("Reading in file..."))
    } else {
        None
    };

    let df = read_file(&args.path, Some(args.headers));

    if let Some(s) = &spinner {
        if df.is_ok() {
            s.suspend(|| spinner::print_status("Finished reading file!", true, &s));
        } else {
            s.suspend(|| spinner::print_status("Failed reading file!", false, &s));
        }
    }

    if let Some(s) = &spinner {
        s.finish();
    }

    Ok(())
}
