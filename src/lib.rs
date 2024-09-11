//! # LEADS Crate
//!
//! LEADS is a **L**azy **E**xploratory **A**nalysis **D**ata **S**ummarizer.
//!
//! Writing the same boilerplate exploratory analysis code in a Juptyer notebook or Excel
//! spreadsheet for each new dataset can be tedious. This crate offers functionality for generating
//! automatic exploratory analysis results for a dataset.
//!
//! ## Quickstart
//!
//! TODO
//!
//! ## Direct Dependencies
//!
//! - [chrono-0.4.38](https://docs.rs/chrono/0.4.38/chrono/index.html) used for date and time
//! functionality.
//! - [clap-4.5.8](https://docs.rs/clap/4.5.8/clap/index.html) used for command line argument
//! handling when run in comand line mode.
//! - [colored-2.1.0](https://docs.rs/colored/2.1.0/colored/index.html) used for colored terminal text
//! when run in command line mode.
//! - [indexmap-2.3.0](https://docs.rs/indexmap/2.3.0/indexmap/index.html) used for ordered hash tables
//! for processing data columns in a consistent order.
//! - [indicatif-0.17.8](https://docs.rs/indicatif/0.17.8/indicatif/index.html) used for progress bar
//! functionality when run in command line mode.
//! - [pdfium-render-0.8.22](https://docs.rs/pdfium-render/0.8.22/pdfium_render/index.html) used for Rust
//! bindings to Pdfium for creating PDF reports.
//! - [polars-0.41.3](https://docs.rs/polars/0.41.3/polars/index.html) used for performing
//! operations on the dataset.
//!   - Opt-in features:
//!     - [polars-lazy-0.41.3](https://docs.rs/polars-lazy/0.41.3/polars_lazy/index.html) for the
//!   Polars lazy API.
//!     - [polars-parquet-0.41.3](https://docs.rs/polars-parquet/0.41.3/polars_parquet/index.html) for support for reading parquet files.
//!     - **moment** for kurtosis and skew statistics.
//!     - **dtype-array** for array data types.
//!     - **random** for random sampling of the dataset.
//! - [thiserror-1.0.63](https://docs.rs/thiserror/1.0.63/thiserror/index.html) for defining library errors.
//! - [plotters-0.3.7](https://docs.rs/plotters/latest/plotters/) for generating visualizations.

use thiserror::Error;

/// LEADS result type alias, used for handling results throughout the crate.
///
/// It returns either a value of type `T` or a `LeadsError`.
pub type LeadsResult<T> = std::result::Result<T, LeadsError>;

/// High level errors in the LEADS crate.
#[derive(Error, Debug)]
pub enum LeadsError {
    /// Error related to input/output operations.
    #[error("IO error -> {0}")]
    IOError(#[from] std::io::Error),

    /// Errors from the base module.
    #[error("Data error -> {0}")]
    Data(#[from] data::base::DataError),

    /// Errors from the PDF report module.
    #[error("Report error -> {0}")]
    Report(#[from] report::pdf::PdfError),

    /// Errors from the descriptive analysis module.
    #[error("Descriptive analysis error -> {0}")]
    DescriptiveAnalysis(#[from] data::descriptive::DescriptiveError),

    /// Errors from the missing values analysis module. 
    #[error("Missing values analysis error -> {0}")]
    MissingValuesAnalysis(#[from] data::missing_values::MissingValueError),

    /// Errors from the visualiztion module.
    #[error("Visualizations error -> {0}")]
    VisualizationError(#[from] data::visualizations::VisualizationError),
}

pub mod data;

pub mod report {
    pub mod glossary;
    pub mod pdf;
}

pub mod spinner;

pub mod prelude {
    pub use crate::data::base::DataInfo;
    pub use crate::data::descriptive::DescriptiveAnalysis;
    pub use crate::data::missing_values::MissingValueAnalysis;
    pub use crate::data::visualizations::VisualizationManager;
    pub use crate::report::pdf::PageManager;
    pub use crate::{LeadsError, LeadsResult};
    /// Re-exports.
    pub use pdfium_render::pdfium::Pdfium;
}
