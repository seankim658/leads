//! # Load Module
//!
//! This module handles the data load into a Polars dataframe.
//!
//! ## Examples
//! ```
//! use leads::prelude::*;
//! use std::env;
//! use std::path::PathBuf;
//! 
//! let project_root = env::var("CARGO_MANIFEST_DIR").unwrap();
//! let mut path = PathBuf::from(project_root);
//! path.push("examples");
//! path.push("data");
//! path.push("pokemon");
//! path.set_extension("csv");
//! 
//! let df = read_file(&path, Some(true)).unwrap();
//! assert_eq!(df.height(), 800);
//! assert_eq!(df.width(), 13);
//! ```

use polars::prelude::*;
use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;
use thiserror::Error;

/// The error types for the data load module.
#[derive(Error, Debug)]
pub enum LoadError {
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
    #[error("Error in parsing file extension, {0}, check file path")]
    FileExtension(String),
}

/// Entry point for loading the data.
///
/// ### Arguments
///
/// - `path`: File path to the file to load into a dataframe.
/// - `headers`: Whether the file has a header row.
///
/// ### Returns
///
/// - `Result<Dataframe, LoadError>`: The resulting dataframe or the load error that occurred.
///
pub fn read_file(path: &PathBuf, headers: Option<bool>) -> Result<DataFrame, LoadError> {
    let headers = match headers {
        Some(headers) => headers,
        None => true,
    };
    match path.extension().and_then(OsStr::to_str) {
        Some("csv") => read_csv(path, headers),
        Some("tsv") => read_tsv(path, headers),
        Some("parquet") => read_parquet(path),
        Some(ext) => Err(LoadError::FileExtension(ext.to_owned())),
        None => Err(LoadError::UnsupportedFormat("No file extension".to_owned())),
    }
}

fn read_csv(path: &PathBuf, headers: bool) -> Result<DataFrame, LoadError> {
    let df = CsvReadOptions::default()
        .with_has_header(headers)
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;
    Ok(df)
}

fn read_tsv(path: &PathBuf, headers: bool) -> Result<DataFrame, LoadError> {
    let df = CsvReadOptions::default()
        .with_parse_options(CsvParseOptions::default().with_separator(b'\t'))
        .with_has_header(headers)
        .try_into_reader_with_file_path(Some(path.into()))?
        .finish()?;
    Ok(df)
}

fn read_parquet(path: &PathBuf) -> Result<DataFrame, LoadError> {
    let df = ParquetReader::new(File::open(path)?).finish()?;
    Ok(df)
}
