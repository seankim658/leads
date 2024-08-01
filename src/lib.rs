//! # LEADS Crate
//!
//!
//! ## Quickstart

use thiserror::Error;

/// LEADS result type alias.
pub type LeadsResult<T> = std::result::Result<T, LeadsError>;

#[derive(Error, Debug)]
pub enum LeadsError {
    #[error("Data error: {0}")]
    Data(#[from] data::base::DataError),

    #[error("Report error: {0}")]
    Report(#[from] report::pdf::PdfError),
}

pub mod data {
    pub mod base;
}

pub mod report {
    pub mod pdf;
}

pub mod spinner;

pub mod prelude {
    pub use crate::{LeadsResult, LeadsError};
    pub use crate::data::base::DataInfo;
    pub use crate::report::pdf::PageManager;
    /// Re-exports.
    pub use pdfium_render::pdfium::Pdfium;
}
