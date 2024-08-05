//! # LEADS Crate
//!
//!
//! ## Quickstart

use thiserror::Error;

/// LEADS result type alias.
pub type LeadsResult<T> = std::result::Result<T, LeadsError>;

#[derive(Error, Debug)]
pub enum LeadsError {
    #[error("Data error -> {0}")]
    Data(#[from] data::base::DataError),

    #[error("Report error -> {0}")]
    Report(#[from] report::pdf::PdfError),

    #[error("Descriptive analysis error -> {0}")]
    DescriptiveAnalysis(#[from] data::descriptive::DescriptiveError)
}

pub mod data {
    pub mod base;
    pub mod descriptive;
}

pub mod report {
    pub mod pdf;
    pub mod glossary;
}

pub mod spinner;

pub mod prelude {
    pub use crate::{LeadsResult, LeadsError};
    pub use crate::data::base::DataInfo;
    pub use crate::report::pdf::PageManager;
    pub use crate::data::descriptive::DescriptiveAnalysis;
    /// Re-exports.
    pub use pdfium_render::pdfium::Pdfium;
}
