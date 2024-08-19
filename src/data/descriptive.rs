//! # Descriptive Analysis Module
//!
//! This module handles the calculation and storage of basic descriptive analysis results for datasets.
//! It provides structures and methods to compute and access various statistical measures for numerical
//! features in a given dataset.
//!
//! ## Design Overview
//!
//! The module is built around two main structures:
//! - `DescriptiveAnalysis`: Represents the overall analysis results for a dataset.
//! - `FeatureStats`: Holds the detailed statistical measures for all numerical features.
//!
//! ### DescriptiveAnalysis Structure
//!
//! `DescriptiveAnalysis` serves as the main interface for accessing analysis results. It contains:
//! - Basic dataset information (number of rows and columns)
//! - A `FeatureStats` instance holding detailed statistics
//! - Mapping structures to efficiently access specific statistics for each feature
//!
//! ### FeatureStats Design
//!
//! `FeatureStats` is implemented as a wrapper around a Polars DataFrame with a specific structure,
//! it contains only one row, where each column represents a specific statistic for a feature.
//!
//! The single-row structure has certain characteristics:
//! 1. It leverages Polars' columnar storage, which may affect performance and memory usage
//! 2. Allows for vectorized operations across all features simultaneously
//! 3. Requires additional mapping structures for data access
//!
//! To navigate this structure, two mapping structures are used:
//! - `column_map`: Maps statistic names to their respective column indices
//! - `feature_indices`: Maps feature names to their starting column index in the DataFrame
//!
//! This approach enables O(1) access to any specific statistic for any feature, given the
//! current DataFrame structure.
//!
//! Note: The efficacy of this design may vary depending on specific use cases and dataset
//! characteristics. Future optimizations might involve reassessing this structure based on
//! performance profiling and specific application needs.
//!
//! ### Efficient Access to Statistics
//!
//! To provide fast access to specific statistics without multiple DataFrame lookups, the module uses:
//! - `column_map`: Maps statistic names to column indices in the FeatureStats DataFrame
//! - `feature_indices`: Maps feature names to row indices in the FeatureStats DataFrame
//!
//! This approach allows for O(1) access to any specific statistic for any feature.
//!
//! ## Usage
//!
//! To perform descriptive analysis on a dataset:
//! 1. Create a `DescriptiveAnalysis` instance using a LazyFrame and its schema
//! 2. Access overall dataset information directly from the `DescriptiveAnalysis` instance
//! 3. Use provided methods to retrieve specific statistics for features of interest
//!
//! ## Error Handling
//!
//! The module uses a custom `DescriptiveError` enum to handle various error scenarios, including:
//! - Polars operation failures
//! - Invalid column or index access attempts
//! - Data type conversion issues
//!
//! ## Examples
//! TODO
//! ```
//! ```

use indexmap::IndexMap;
use polars::{lazy::dsl::*, prelude::*};
use thiserror::Error;

/// The error types for the descriptive analysis module.
#[derive(Error, Debug)]
pub enum DescriptiveError {
    /// Occurs when Polars fails to complete an operation.
    #[error("Polars error: {0}")]
    Polars(#[from] PolarsError),

    /// Occurs during schema validation when creating the descriptive analysis FeatureStats struct.
    #[error("FeatureStats schema error: {0}")]
    Schema(String),

    /// Occurs when trying to interact with a column that doesn't exist.
    #[error("Non-existent column: {0}")]
    InvalidCol(String),

    /// Occurs when trying to access and invalid column index
    #[error("Invalid index: {0}")]
    InvalidIndex(String),

    /// Occurs when data type conversion fails for a column value.
    #[error("Invalid data conversion for column {0}, from {1} to {2}")]
    InvalidConversion(String, String, String),
}

/// Struct to hold the overall descriptive analysis results.
#[derive(Debug)]
pub struct DescriptiveAnalysis {
    /// The number of rows in the data.
    pub n_rows: u64,
    /// The number of columns in the data.
    pub n_cols: u64,
    /// The map of each feature's descriptive analysis results.
    pub column_stats: FeatureStats,
    /// The column offset map for accessing specific statistics.
    pub column_map: IndexMap<String, usize>,
    /// Offset indices for each feature in the FeatureStats Dataframe.
    pub feature_indices: IndexMap<String, usize>,
}

impl DescriptiveAnalysis {
    /// Constructor for the DescriptiveAnalysis struct.
    ///
    /// ### Parameters
    ///
    /// - `lazy_df`: Reference to the LazyFrame.
    /// - `schema`: Reference to the lazy frame's schema.
    ///
    /// ### Returns
    ///
    /// - `Result<Self, DescriptiveError>`: A new DescriptiveAnalysis instance or an error.
    ///
    /// ### Errors
    ///
    /// This method can return a DescriptiveError if:
    /// - There's an issue with Polars operations.
    /// - There are no numeric columns in the dataset.
    /// - There's an issue accessing the computed statistics.
    pub fn new(lazy_df: &LazyFrame, schema: &Schema) -> Result<Self, DescriptiveError> {
        let n_cols = schema.len() as u64;
        let numeric_columns: Vec<String> = schema
            .iter()
            .filter(|(_, dtype)| dtype.is_numeric())
            .map(|(name, _)| name.to_string())
            .collect();

        let stats_df = lazy_df
            .clone()
            .select(
                numeric_columns
                    .iter()
                    .flat_map(|col_name| {
                        vec![
                            lit(col_name.to_owned()).alias(&format!("{}", col_name)),
                            col(col_name).min().alias(&format!("{}_min", col_name)),
                            col(col_name).max().alias(&format!("{}_max", col_name)),
                            col(col_name).mean().alias(&format!("{}_mean", col_name)),
                            col(col_name)
                                .median()
                                .alias(&format!("{}_median", col_name)),
                            col(col_name).std(1).alias(&format!("{}_std_dev", col_name)),
                            col(col_name)
                                .quantile(lit(0.25), QuantileInterpolOptions::Linear)
                                .alias(&format!("{}_q1", col_name)),
                            col(col_name)
                                .quantile(lit(0.75), QuantileInterpolOptions::Linear)
                                .alias(&format!("{}_q3", col_name)),
                            (col(col_name).quantile(lit(0.75), QuantileInterpolOptions::Linear)
                                - col(col_name)
                                    .quantile(lit(0.75), QuantileInterpolOptions::Linear))
                            .alias(&format!("{}_iqr", col_name)),
                            col(col_name)
                                .skew(true)
                                .alias(&format!("{}_skew_bias", col_name)),
                            col(col_name)
                                .skew(false)
                                .alias(&format!("{}_skew_raw", col_name)),
                            col(col_name)
                                .kurtosis(true, false)
                                .alias(&format!("{}_kurtosis", col_name)),
                            col(&col_name).count().alias(&format!("{}_count", col_name)),
                        ]
                    })
                    .collect::<Vec<Expr>>(),
            )
            .collect()?;

        let feature_stats = FeatureStats::new(stats_df)?;

        let column_map: IndexMap<String, usize> = IndexMap::from([
            ("column_name".to_owned(), 0),
            ("min".to_owned(), 1),
            ("max".to_owned(), 2),
            ("mean".to_owned(), 3),
            ("median".to_owned(), 4),
            ("std_dev".to_owned(), 5),
            ("q1".to_owned(), 6),
            ("q3".to_owned(), 7),
            ("iqr".to_owned(), 8),
            ("skewness_bias".to_owned(), 9),
            ("skewness_raw".to_owned(), 10),
            ("kurtosis".to_owned(), 11),
            ("count".to_owned(), 12),
        ]);

        let feature_indices: IndexMap<String, usize> = numeric_columns
            .iter()
            .enumerate()
            .map(|(index, name)| (name.clone(), index * column_map.len()))
            .collect();

        let n_rows = feature_stats.get_count(
            numeric_columns
                .get(0)
                .ok_or_else(|| DescriptiveError::InvalidIndex(format!("0")))?,
            &feature_indices,
            &column_map,
        )?;

        Ok(Self {
            n_rows,
            n_cols,
            column_stats: feature_stats,
            column_map,
            feature_indices,
        })
    }
}

/// Struct to hold descriptive analysis results for all features.
#[derive(Debug)]
pub struct FeatureStats(DataFrame);

impl FeatureStats {
    /// Constructor for the FeatureStats struct.
    ///
    /// ### Parameters
    ///
    /// - `df`: Dataframe to convert to a FeatureStats struct.
    pub fn new(df: DataFrame) -> Result<Self, DescriptiveError> {
        Ok(Self(df))
    }

    /// Gets a vector of all the descriptive analysis values for each feature.
    ///
    /// ### Parameters
    ///
    /// - `feature_indices`: The map of offsets for each feature in the DataFrame.
    /// - `column_map`: The map of offsets for each descriptive analysis metric.
    ///
    /// ### Returns
    ///
    /// - `Result<Vec<IndexMap<String, String>>, DescriptiveError>`: A vector of IndexMaps,
    ///   each containing the statistics for a feature, or an error.
    ///
    /// ### Errors
    ///
    /// This method can return a DescriptiveError if:
    /// - The DataFrame is empty.
    /// - There's an issue accessing a specific statistic.
    pub fn get_analysis_values(
        &self,
        feature_indices: &IndexMap<String, usize>,
        column_map: &IndexMap<String, usize>,
    ) -> Result<Vec<IndexMap<String, String>>, DescriptiveError> {
        let mut result = Vec::with_capacity(feature_indices.len());

        let row = self
            .0
            .get(0)
            .ok_or_else(|| DescriptiveError::Schema("No data".to_owned()))?;

        for (_, feature_index) in feature_indices {
            let mut feature_stats = IndexMap::with_capacity(column_map.len());

            for (statistic, stat_offset) in column_map {
                let column_index = feature_index + stat_offset;
                let value = row.get(column_index).ok_or_else(|| {
                    DescriptiveError::InvalidIndex(format!(
                        "attempted to index FeatureStats Vec with index {}",
                        column_index.to_string()
                    ))
                })?;
                feature_stats.insert(statistic.to_owned(), value.to_string());
            }
            result.push(feature_stats);
        }

        Ok(result)
    }

    /// Gets the row count for a specific feature.
    ///
    /// ### Parameters
    ///
    /// - `feature`: The name of the feature.
    /// - `feature_indices`: The map of offsets for each feature in the DataFrame.
    /// - `column_map`: The map of offsets for each descriptive analysis metric.
    ///
    /// ### Returns
    ///
    /// - `Result<u64, DescriptiveError>`: The count as a u64, or an error.
    ///
    /// ### Errors
    ///
    /// This method can return a DescriptiveError if:
    /// - The feature doesn't exist.
    /// - The count cannot be converted to a u64.
    pub fn get_count(
        &self,
        feature: &str,
        feature_indices: &IndexMap<String, usize>,
        column_map: &IndexMap<String, usize>,
    ) -> Result<u64, DescriptiveError> {
        let value = self.get_statistic(feature, "count", feature_indices, column_map)?;
        match value {
            AnyValue::UInt64(count) => Ok(count),
            AnyValue::UInt32(count) => Ok(count as u64),
            _ => Err(DescriptiveError::InvalidConversion(
                "count".to_owned(),
                value.dtype().to_string(),
                "u64".to_owned(),
            )),
        }
    }

    /// Get a single statistic for a feature.
    fn get_statistic(
        &self,
        feature: &str,
        statistic: &str,
        feature_indices: &IndexMap<String, usize>,
        column_map: &IndexMap<String, usize>,
    ) -> Result<AnyValue, DescriptiveError> {
        let feature_index = feature_indices
            .get(feature)
            .ok_or_else(|| DescriptiveError::InvalidCol(feature.to_owned()))?;
        let statistic_offset = column_map
            .get(statistic)
            .ok_or_else(|| DescriptiveError::InvalidCol(feature.to_owned()))?;
        let column_index = feature_index + statistic_offset;

        // TODO : there should be a better way of doing that avoids a .get() call.
        let value = self
            .0
            .get(0)
            .ok_or_else(|| DescriptiveError::InvalidCol("No data".to_owned()))?
            .get(column_index)
            .ok_or_else(|| DescriptiveError::InvalidIndex(format!("{}", column_index)))?
            .to_owned();

        Ok(value)
    }
}
