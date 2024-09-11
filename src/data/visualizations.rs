//! Visualization Module
//!
//! This module serves as the entry point for generating visualizations using the 
//! [plotters](https://docs.rs/plotters/0.3.7/plotters/) crate. It manages the creation
//! and organization of various plot types.

use super::viz_lib::missing_value_viz;
use crate::data::missing_values::MissingValueAnalysis;
use polars::prelude::*;
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

/// The error types for the visualization module.
#[derive(Error, Debug)]
pub enum VisualizationError {
    /// Occurs when creating the missing values plots fails.
    #[error("Missing values plot error: {0}")]
    MissingValuesPlotting(#[from] crate::data::viz_lib::missing_value_viz::MissingValuesPlotError),
}

/// Enum to represent which section each visualization corresponds to.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum ReportSection {
    /// The missing values analysis section.
    MissingValues,
}

/// Manages the creation and storage of visualizations for different report sections.
///
/// This struct organizes visualizations by report section, storing the title and file path
/// of each generated plot.
#[derive(Debug)]
pub struct VisualizationManager {
    /// A nested `HashMap` structure where:
    /// - The outer `HashMap` maps `ReportSection` to an inner `HashMap`.
    /// - The inner `HashMap` maps visualization titles to their file paths.
    pub visualizations: HashMap<ReportSection, HashMap<String, PathBuf>>,
}

impl VisualizationManager {
    /// Creates a new `VisualizationManager` and generates all required visualizations.
    ///
    /// ### Parameters
    ///
    /// - `plot_dir`: Directory where plot images will be saved.
    /// - `lazy_df`: The `LazyFrame` containing the dataset to visualize.
    /// - `shape`: The shape of the dataset (rows, columns).
    /// - `missing_values_analysis`: Analysis results for missing values.
    ///
    /// ### Returns
    ///
    /// - `Result<Self, VisualizationError>`: A new `VisualizationManager` instance or an error.
    pub fn new(
        plot_dir: &PathBuf,
        lazy_df: &LazyFrame,
        shape: (u64, u64),
        missing_values_analysis: &MissingValueAnalysis,
    ) -> Result<Self, VisualizationError> {
        let mut visualizations: HashMap<ReportSection, HashMap<String, PathBuf>> = HashMap::new();

        // Generate missing values heatmap.
        let missing_value_heatmap = missing_value_viz::build_missing_data_matrix(
            lazy_df,
            missing_values_analysis,
            plot_dir,
        )?;
        let missing_value_plots = HashMap::from([missing_value_heatmap]);
        visualizations.insert(ReportSection::MissingValues, missing_value_plots);

        Ok(Self { visualizations })
    }
}
