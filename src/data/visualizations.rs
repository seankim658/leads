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

/// Enum for the sample of the dataset to generate visualizations for.
pub enum SampleModeEnum {
    /// Use a strict limit for sampling of the data.
    Limit(u64),
    /// Use a certain ratio for sampling of the data.
    Ratio(f64),
    /// Use the full dataset.
    Full,
}

/// The error types for the visualization module.
#[derive(Error, Debug)]
pub enum VisualizationError {
    /// Occurs when sampling the lazy frame fails.
    #[error("Error sampling the dataframe: {0}")]
    DataFrameSamplingError(String),

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
        sampling_mode: SampleModeEnum,
    ) -> Result<Self, VisualizationError> {
        let mut visualizations: HashMap<ReportSection, HashMap<String, PathBuf>> = HashMap::new();

        let df = sample_dataframe(lazy_df, sampling_mode)?;

        // Generate missing values visualizations.
        let missing_value_plots = missing_value_viz::build_all_visualizations(&df, missing_values_analysis, plot_dir)?;
        visualizations.insert(ReportSection::MissingValues, missing_value_plots);

        Ok(Self { visualizations })
    }
}

/// Collects a lazy frame into a dataframe and applies the sampling if applicable.
///
/// Type of sampling depends on the global `SAMPLE_MODE` setting. TODO : Should this eventually be
/// abstracted into a CLI argument?
///
/// ### Parameters
///
/// - `lazy_df`: Reference to the dataset `LazyFrame`.
///
/// ### Returns
///
/// - `Result<DataFrame, MissingValuesPlotError>`: The collected dataframe or the
/// `MissingValuesPlotError` error.
pub fn sample_dataframe(
    lazy_df: &LazyFrame,
    sampling_mode: SampleModeEnum,
) -> Result<DataFrame, VisualizationError> {
    match sampling_mode {
        SampleModeEnum::Limit(limit) => {
            let df = lazy_df
                .clone()
                .select([all().sample_n(lit(limit), false, false, None)])
                .collect()
                .map_err(|e| {
                    VisualizationError::DataFrameSamplingError(format!(
                        "Couldn't collect limit ({}) sampled dataframe: {}",
                        limit, e
                    ))
                })?;
            Ok(df)
        }
        SampleModeEnum::Ratio(ratio) => {
            let df = lazy_df
                .clone()
                .select([all().sample_frac(lit(ratio), false, false, None)])
                .collect()
                .map_err(|e| {
                    VisualizationError::DataFrameSamplingError(format!(
                        "Couldn't collect ratio ({}) sampled dataframe: {}",
                        ratio, e
                    ))
                })?;
            Ok(df)
        }
        SampleModeEnum::Full => {
            let df = lazy_df.clone().collect().map_err(|e| {
                VisualizationError::DataFrameSamplingError(format!(
                    "Couldn't collect lazy frame: {}",
                    e
                ))
            })?;
            Ok(df)
        }
    }
}
