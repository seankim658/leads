//! # Base Data Module
//!
//! This module handles the data load into a Polars dataframe.
//! TODO : clean this up
//! ## Examples
//! ```
//! ```

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

/// Struct to hold the data information.
pub struct DataInfo {
    /// Name of the dataset (inferred from the file name).
    pub data_title: String,
    /// The column names (or indices) of the dataset.
    pub column_types: IndexMap<String, DataType>,
    /// The polars dataframe of the data.
    pub data: LazyFrame,
}

impl DataInfo {
    /// Constructor for the DataInfo struct.
    ///
    /// ### Parameters
    /// - `path`: The path to the data file.
    /// - `headers`: Whether there is a header row in the data file.
    ///
    /// ### Returns
    ///
    /// - `Result<Self, DataError>`: The DataInfo struct or a propagated Dataerror.
    ///
    pub fn new(path: &PathBuf, headers: Option<bool>) -> Result<Self, DataError> {
        let headers = headers.unwrap_or(true);
        let mut lazy_df = read_file(path, headers)?;

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
                return Err(DataError::DuplicateHeader(column_name.clone()));
            }
        }

        Ok(DataInfo {
            data_title,
            column_types,
            data: lazy_df,
        })
    }
}

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
