//! # Missing Values Module
//!
//! This module handles the detection of missing values in the dataset.

use indexmap::IndexMap;
use polars::{lazy::dsl::*, prelude::*};
use thiserror::Error;

/// The error types for the missing values module.
#[derive(Error, Debug)]
pub enum MissingValueError {
    /// Error raised when an operation fails due to Polars-specific issues.
    #[error("Polars error: {0}")]
    Polars(#[from] PolarsError),

    /// Error raised when attempting to access a column that doesn't exist.
    #[error("Non-existent column: {0}")]
    InvalidCol(String),
}

/// Holds the results of missing value analysis for each column in a dataset.
#[derive(Debug)]
pub struct MissingValueAnalysis {
    /// A map where each key is a column name and the value is a tuple containing:
    /// - the count of missing values
    /// - the percentage of missing values relative to the total number of rows
    pub column_missing_values: IndexMap<String, (u64, f64)>,
}

impl MissingValueAnalysis {
    /// Creates a new `MissingValueAnalysis` by calculating the number of missing values
    /// for each column in the dataset.
    ///
    /// # Parameters
    ///
    /// * `lazy_df` - A reference to the LazyFrame representing the dataset to analyze.
    /// * `schema` - The schema of the dataset, used to identify the columns.
    /// * `n_rows` - The total number of rows in the dataset, used to calculate percentages.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the analysis results (`MissingValueAnalysis`)
    /// or an error (`MissingValueError`) if an operation fails.
    ///
    /// # Errors
    ///
    /// This function will return `MissingValueError::Polars` if Polars fails during an operation,
    /// or `MissingValueError::InvalidCol` if it tries to analyze a column that doesn't exist.
    pub fn new(
        lazy_df: &LazyFrame,
        schema: &Schema,
        n_rows: u64,
    ) -> Result<Self, MissingValueError> {
        // Initialize the missing values map.
        let mut column_missing_values: IndexMap<String, (u64, f64)> = IndexMap::new();

        // Iterate through each column in the schema to check for missing values.
        for field in schema.iter_fields() {
            let column_name = field.name().to_string();

            // Lazily count the number of null values for the current column.
            let missing_count_expr = lazy_df.clone().select([col(column_name.as_str())
                .is_null()
                .sum()
                .cast(DataType::UInt64)
                .alias("missing_count")]);

            // Collect the missing count results.
            let missing_count_df = missing_count_expr.collect()?;
            let missing_count = missing_count_df
                .column("missing_count")?
                .u64()?
                .get(0)
                .unwrap_or(0);

            // Calculate the percentage of missing values for the column.
            let missing_percentage = (missing_count as f64 / n_rows as f64) * 100.0;

            // Store the results in the map.
            column_missing_values.insert(column_name, (missing_count, missing_percentage));
        }

        Ok(MissingValueAnalysis {
            column_missing_values,
        })
    }
}
