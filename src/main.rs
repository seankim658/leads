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

    /// Path to the directory to save the output report and the optional visualizations.
    #[arg()]
    output_path: PathBuf,

    /// Indicates whether the file has headers. Absence indicates True.
    #[arg(short = 'r', long, action(ArgAction::SetFalse))]
    headers: bool,

    /// Toggle visualization generation. Absence indicates False.
    #[arg(long, action(ArgAction::SetTrue))]
    visualizations: bool,

    /// Whether a progress spinner and status messages should be printed. Absence indicates False.
    #[arg(long, action(ArgAction::SetTrue))]
    verbose: bool,
}

fn main() -> LeadsResult<()> {
    let args = Args::parse();

    // Ensure the output directory exists.
    let output_dir = &args.output_path;
    std::fs::create_dir_all(output_dir)?;

    let spinner = if args.verbose {
        Some(spinner::setup_spinner("Reading in file..."))
    } else {
        None
    };

    // Create the visulizations directory if needed.
    let plots_dir = if args.visualizations {
        let plots_dir = output_dir.join("plots");
        std::fs::create_dir_all(&plots_dir)?;
        Some(plots_dir)
    } else {
        None
    };

    // Read in data.
    let data = handle_operation(
        || DataInfo::new(&args.path, Some(args.headers), &plots_dir),
        "Finished reading file!",
        "Failed reading file!",
        &spinner,
    )?;
    
    // Extract and format the dataset name for the report name.
    let report_filename = format!("{}_report.pdf", data.data_title.replace(" ", "_"));
    let report_path = output_dir.join(report_filename);

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
        || page_manager.generate_report(&data),
        "Finished report generation.",
        "Failed to generate report.",
        &spinner,
    )?;

    // Save report.
    handle_operation(
        || page_manager.save_to_file(&report_path),
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
