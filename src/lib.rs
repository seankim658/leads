//! # LEADS Crate
//!
//! 
//! ## Quickstart

use thiserror::Error;

/// LEADS result type alias.
pub type LeadsResult<T> = std::result::Result<T, LeadsError>;

#[derive(Error, Debug)]
pub enum LeadsError {
    #[error("Data loading error: {0}")]
    Load(#[from] data::load::LoadError),
}

pub mod data {
    pub mod load;
}

pub mod spinner;

pub mod prelude {
    pub use crate::{LeadsResult, LeadsError};
    pub use crate::data::load::read_file; 
}
