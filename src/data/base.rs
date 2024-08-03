//! # Base Data Module
//!
//! This module handles the data load into a Polars dataframe.
//! TODO : clean this up
//! ## Examples
//! ```
//! ```

use polars::prelude::*;
use indexmap::IndexMap;
use std::ffi::OsStr;
use std::fs::File;
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
    Polars(#[from] polars::error::PolarsError),

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
    pub data: DataFrame,
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
        let headers = match headers {
            Some(headers) => headers,
            None => true,
        };
        let data = read_file(path, headers)?;

        let data_title = path
            .as_path()
            .file_stem()
            .and_then(|v| v.to_str())
            .map(|s| s.to_owned())
            .ok_or_else(|| {
                DataError::FilenameParse(path.to_str().unwrap_or_default().to_owned())
            })?;

        let dtypes = data.dtypes();
        let mut column_types: IndexMap<String, DataType> = IndexMap::new();
        for (index, &col) in data.get_column_names().iter().enumerate() {
            let col_name = col.to_owned();
            if column_types.contains_key(&col_name) {
                return Err(DataError::DuplicateHeader(col_name));
            }
            column_types.insert(col_name, dtypes[index].to_owned());
        }

        Ok(DataInfo {
            data_title,
            column_types,
            data,
        })
    }
}

fn read_file(path: &PathBuf, headers: bool) -> Result<DataFrame, DataError> {
    match path.extension().and_then(OsStr::to_str) {
        Some("csv") => read_csv(path, headers),
        Some("tsv") => read_tsv(path, headers),
        Some("parquet") => read_parquet(path),
        Some(ext) => Err(DataError::FileExtension(ext.to_owned())),
        None => Err(DataError::UnsupportedFormat("No file extension".to_owned())),
    }
}

fn read_csv(path: &PathBuf, headers: bool) -> Result<DataFrame, DataError> {
    let df = CsvReadOptions::default()
        .with_has_header(headers)
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;
    Ok(df)
}

fn read_tsv(path: &PathBuf, headers: bool) -> Result<DataFrame, DataError> {
    let df = CsvReadOptions::default()
        .with_parse_options(CsvParseOptions::default().with_separator(b'\t'))
        .with_has_header(headers)
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;
    Ok(df)
}

fn read_parquet(path: &PathBuf) -> Result<DataFrame, DataError> {
    let df = ParquetReader::new(File::open(path)?).finish()?;
    Ok(df)
}
