//! # Visualization Module
//!
//! This module handles generating the Visualizations using the Python backend.

use pyo3::prelude::*;
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

/// The error types for the visualization module.
#[derive(Error, Debug)]
pub enum VisualizationError {
    
    /// Occurs when the Python backend fails to complete an operation.
    #[error("Python Error: {0}")]
    PythonError(String),
}

/// Enum to represent which section each visualization corresponds to.
#[derive(Debug)]
pub enum ReportSection {
    /// The missing values analysis section.
    MissingValues,
}

/// Struct which holds the filepaths to the generated visualizations.
#[derive(Debug)]
pub struct VisualizationManager {
    /// Outter hashmap represents the report section that the visualizations correspond to and the
    /// inner hashmap holds the visualization's title and path to the image file.
    pub visualizations: HashMap<ReportSection, HashMap<String, PathBuf>>,
}

impl VisualizationManager {
    /// Constructor for the VisualizationManager struct.
    ///
    /// ### Parameters
    ///
    /// - `path`: Path to the data file.
    pub fn new(path: &PathBuf) -> Result<Self, VisualizationError> {
        // TODO
    }
}
