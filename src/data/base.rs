//! # Base Data Module
//!
//! This module handles loading data into a Polars LazyFrame from various file formats.
//! It provides functionality to read CSV, TSV, and Parquet files, and performs initial
//! data processing and analysis.
//!
//! TODO : clean this up
//! ## Examples
//! ```
//! ```

use crate::{
    data::{
        descriptive::DescriptiveAnalysis, missing_values::MissingValueAnalysis,
        visualizations::VisualizationManager,
    },
    LeadsError,
};
use indexmap::IndexMap;
use polars::prelude::*;
use std::ffi::OsStr;
use std::path::PathBuf;
use thiserror::Error;

/// The error types for the base data module.
#[derive(Error, Debug)]
pub enum DataError {
    /// Occurs when an I/O operation fails.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Occurs when Polars fails to complete an operation.
    #[error("Polars error: {0}")]
    Polars(#[from] PolarsError),

    /// Occurs when Polars fails to lazy infer the schema.
    #[error("Polars schema error: {0}")]
    PolarsSchema(String),

    /// Occurs when an unsupported file format is passed.
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    /// Occurs when the file extension cannot be parsed.
    #[error("Error in parsing file extension, check file path: {0}")]
    FileExtension(String),

    /// Occurs when the file name cannot be parsed.
    #[error("Error parsing file name from path: {0}")]
    FilenameParse(String),

    /// Occurs when duplicate column headers are detected.
    #[error("Duplicate column name detected: {0}")]
    DuplicateHeader(String),
}

/// Struct to hold the data information, analysis results, and analysis metadata.
pub struct DataInfo {
    /// Name of the dataset (inferred from the file name).
    pub data_title: String,
    /// Map of column names to their data types.
    pub column_types: IndexMap<String, DataType>,
    /// The Polars LazyFrame containing the data.
    pub data: LazyFrame,
    /// The descriptive analysis results for the dataset.
    pub descriptive_analysis: DescriptiveAnalysis,
    /// The missing values analysis results for the dataset.
    pub missing_value_analysis: MissingValueAnalysis,
    /// The visualization results (if applicable) for the dataset.
    pub visualizations: Option<VisualizationManager>,
}

impl DataInfo {
    /// Constructs a new DataInfo instance by reading and analyzing a data file.
    ///
    /// ### Parameters
    /// - `path`: The path to the data file.
    /// - `headers`: Optional boolean indicating whether the file has headers. Defaults to true if not provided.
    /// - `plot_dir`: Optional plot directory to store the plots. If user ran with visualizations
    /// this will be the directory path, otherwise is `None`.
    ///
    /// ### Returns
    /// - `Result<Self, LeadsError>`: A new DataInfo instance or an error.
    ///
    /// ### Errors
    /// This method can return a LeadsError if:
    /// - The file cannot be read or parsed.
    /// - The file format is unsupported.
    /// - There are duplicate column headers.
    /// - The descriptive analysis fails.
    pub fn new(
        path: &PathBuf,
        headers: Option<bool>,
        plot_dir: Option<&PathBuf>,
    ) -> Result<Self, LeadsError> {
        let headers = headers.unwrap_or(true);

        // Get the file extension and process accordingly.
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| DataError::FilenameParse("No file extension found".to_owned()))?;
        let mut lazy_df = match extension {
            "csv" => read_csv(path, headers),
            "tsv" => read_tsv(path, headers),
            "parquet" => read_parquet(path),
            _ => Err(DataError::UnsupportedFormat(format!(
                "Unsupported file format: {}",
                extension
            ))),
        }?;

        let data_title = path
            .as_path()
            .file_stem()
            .and_then(|v| v.to_str())
            .map(|s| s.to_owned())
            .ok_or_else(|| {
                DataError::FilenameParse(path.to_str().unwrap_or_default().to_owned())
            })?;

        let schema = lazy_df
            .schema()
            .map_err(|e| DataError::PolarsSchema(format!("Unable to infer data schema: {}", e)))?;
        let column_types: IndexMap<String, DataType> = schema
            .iter()
            .map(|(name, dtype)| (name.to_string(), dtype.clone()))
            .collect();

        // Check for duplicate headers.
        let mut seen_columns = std::collections::HashSet::new();
        for column_name in column_types.keys() {
            if !seen_columns.insert(column_name) {
                Err(DataError::DuplicateHeader(column_name.clone()))?
            }
        }

        let descriptive_analysis = DescriptiveAnalysis::new(&lazy_df, &schema)?;
        let missing_value_analysis =
            MissingValueAnalysis::new(&lazy_df, &schema, descriptive_analysis.n_rows)?;

        let visualization_manager = if plot_dir.is_some() {
            Some(VisualizationManager::new(path, extension, plot_dir.unwrap())?)
        } else {
            None
        };

        Ok(DataInfo {
            data_title,
            column_types,
            data: lazy_df,
            descriptive_analysis,
            missing_value_analysis,
            visualizations: visualization_manager,
        })
    }
}

/// Reads a file and returns a LazyFrame based on the file extension.
///
/// ### Parameters
/// - `path`: The path to the file.
/// - `headers`: Boolean indicating whether the file has headers.
///
/// ### Returns
/// - `Result<LazyFrame, DataError>`: A LazyFrame containing the file data or an error.
///
/// ### Errors
/// This function can return a DataError if:
/// - The file extension is unsupported or missing.
/// - The file cannot be read or parsed.
#[deprecated(since="0.0.1", note="File readers are used directly instead of through this mapping function.")]
fn read_file(path: &PathBuf, headers: bool) -> Result<LazyFrame, DataError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("csv") => read_csv(path, headers),
        Some("tsv") => read_tsv(path, headers),
        Some("parquet") => read_parquet(path),
        Some(ext) => Err(DataError::FileExtension(ext.to_owned())),
        None => Err(DataError::UnsupportedFormat("No file extension".to_owned())),
    }
}

fn read_csv(path: &PathBuf, headers: bool) -> Result<LazyFrame, DataError> {
    let df = LazyCsvReader::new(path.to_str().unwrap())
        .with_has_header(headers)
        .finish()?;
    Ok(df)
}

fn read_tsv(path: &PathBuf, headers: bool) -> Result<LazyFrame, DataError> {
    let df = LazyCsvReader::new(path.to_str().unwrap())
        .with_has_header(headers)
        .with_separator(b'\t')
        .finish()?;
    Ok(df)
}

fn read_parquet(path: &PathBuf) -> Result<LazyFrame, DataError> {
    let df = LazyFrame::scan_parquet(path.to_str().unwrap(), Default::default())?;
    Ok(df)
}
