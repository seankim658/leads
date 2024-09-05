//! # Missing Values Module
//!
//! This module handles the detection of missing values in the dataset.

use indexmap::IndexMap;
use polars::{lazy::dsl::*, prelude::*};
use thiserror::Error;

/// The error types for the missing values module.
#[derive(Error, Debug)]
pub enum MissingValueError {
    /// Occurs when Polars fails to complete an operation.
    #[error("Polars error: {0}")]
    Polars(#[from] PolarsError),

    /// Occurs when trying to interact with a column that doesn't exist.
    #[error("Non-existent column: {0}")]
    InvalidCol(String),
}

/// Struct to hold the missing value analysis results.
#[derive(Debug)]
pub struct MissingValueAnalysis {
    /// A map containing the missing value count and percentage for each column.
    pub column_missing_values: IndexMap<String, (u64, f64)>,
}

impl MissingValueAnalysis {
    /// Constructor for the MissingValueAnalysis struct.
    ///
    /// ### Parameters
    /// - `lazy_df`: Reference to the LazyFrame.
    /// - `schema`: Reference to the lazy frame's schema.
    /// - `n_rows`: Number of rows for the dataset.
    ///
    /// ### Returns
    /// - `Result<Self, MissingValueError>`: MissingValueAnalysis results or an error.
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
