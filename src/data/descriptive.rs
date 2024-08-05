//! # Descriptive Analysis Module
//!
//! ## Examples
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
    /// The column offset map.
    pub column_map: IndexMap<String, usize>,
    /// Offset indices for each feature.
    pub feature_indices: IndexMap<String, usize>,
}

impl DescriptiveAnalysis {
    /// Constructor for the DescriptiveAnalysis struct.
    ///
    /// ### Parameters
    ///
    /// - `lazy_df`: Reference to the LazyFrame.
    /// - `schema`: Reference to the lazy frame's schema.
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

/// Struct to hold an individual feature's (column) descriptive analysis results.
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

    /// Gets a vector of all the descriptive analysis values for printing.
    ///
    /// ### Parameters
    ///
    /// - `feature`: The feature to get the descriptive analysis data for.
    /// - `feature_indices`: The map of offsets for each feature.
    /// - `column_map`: The map of offsets for each descriptive analysis metric.
    ///
    /// ### Returns
    ///
    /// - `Result<IndexMap<String, String>, DescriptiveError>`: The IndexMap containing each metric
    /// name as a key and the corresponding metric value as a string or the propagated DescriptiveError.
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

    /// Gets the row count as u64.
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
