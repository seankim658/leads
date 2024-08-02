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

    /// Path to save the output report.
    #[arg()]
    output_path: PathBuf,

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

    // Read in data.
    let data = handle_operation(
        || DataInfo::new(&args.path, Some(args.headers)),
        "Finished reading file!",
        "Failed reading file!",
        &spinner,
    )?;

    // TODO Start exploratory analysis.

    // Create page manager.
    let pdfium = Pdfium::default();
    let mut page_manager = handle_operation(
        || PageManager::new(&pdfium),
        "Created report document.",
        "Failed to create report document.",
        &spinner,
    )?;

    // Generate the report.
    handle_operation(
        || page_manager.generate_report(&data.data_title, &data.column_types),
        "Finished report generation.",
        "Failed to generate report.",
        &spinner,
    )?;

    // Save report.
    handle_operation(
        || page_manager.save_to_file(&args.output_path),
        &format!(
            "Report successfully generated and saved to location {}",
            &args.output_path.to_str().unwrap_or_default()
        ),
        &format!(
            "Failed saving report to {}",
            &args.output_path.to_str().unwrap_or_default(),
        ),
        &spinner,
    )?;

    if let Some(s) = &spinner {
        s.finish_with_message("Finished!");
    }

    Ok(())
}

fn handle_operation<T, F, E>(
    operation: F,
    success_message: &str,
    failure_message: &str,
    spinner: &Option<indicatif::ProgressBar>,
) -> LeadsResult<T>
where
    F: FnOnce() -> Result<T, E>,
    LeadsError: From<E>,
{
    match operation() {
        Ok(result) => {
            if let Some(s) = spinner {
                s.suspend(|| spinner::print_status(success_message, true, s));
            }
            Ok(result)
        }
        Err(e) => {
            if let Some(s) = spinner {
                s.suspend(|| spinner::print_status(failure_message, false, s));
            }
            Err(LeadsError::from(e))
        }
    }
}
