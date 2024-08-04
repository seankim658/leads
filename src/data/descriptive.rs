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
    #[error("FeatureStats schema error")]
    Schema(),

    /// Occurs when trying to interact with a column that doesn't exist.
    #[error("Non-existent column: {0}")]
    InvalidCol(String),

    /// Occurs when data type conversion fails for a column value.
    #[error("Invalid data conversion for column {0}, from {1} to {2}")]
    InvalidConversion(String, String, String),

}

/// Struct to hold the overall descriptive analysis results.
pub struct DescriptiveAnalysis {
    /// The number of rows in the data.
    pub n_rows: usize,
    /// The number of columns in the data.
    pub n_cols: usize,
    /// The map of each feature's descriptive analysis results.
    pub column_stats: FeatureStats,
    /// The column map for the FeatureStats struct.
    pub column_map: IndexMap<String, usize>,
}

impl DescriptiveAnalysis {
    /// Constructor for the DescriptiveAnalysis struct.
    ///
    /// ### Parameters
    ///
    /// - `lazy_df`: Reference to the LazyFrame.
    /// - `schema`: Reference to the lazy frame's schema.
    pub fn new(lazy_df: &LazyFrame, schema: &Schema) -> Result<Self, DescriptiveError> {
        let n_cols = schema.len();
        let numeric_columns: Vec<String> = schema
            .iter()
            .filter(|(_, dtype)| dtype.is_numeric())
            .map(|(name, _)| name.to_string())
            .collect();

        let stats_df = lazy_df
            .select(
                numeric_columns
                    .iter()
                    .map(|col_name| {
                        vec![
                            lit(col_name.to_owned()).alias("column_name"),
                            col(col_name).min().alias("min"),
                            col(col_name).max().alias("max"),
                            col(col_name).mean().alias("mean"),
                            col(col_name).median().alias("median"),
                            col(col_name).std(1).alias("std_dev"),
                            col(col_name)
                                .quantile(lit(0.25), QuantileInterpolOptions::Linear)
                                .alias("q1"),
                            col(col_name)
                                .quantile(lit(0.75), QuantileInterpolOptions::Linear)
                                .alias("q3"),
                            (col(col_name).quantile(lit(0.75), QuantileInterpolOptions::Linear)
                                - col(col_name)
                                    .quantile(lit(0.75), QuantileInterpolOptions::Linear))
                            .alias("iqr"),
                            col(col_name).skew(true).alias("skew_bias"),
                            col(col_name).skew(false).alias("skew_raw"),
                            col(col_name).kurtosis(true, false).alias("kurtosis"),
                            col(&col_name).count().alias("count"),
                        ]
                    })
                    .collect(),
            )
            .collect()?;

        let column_stats = FeatureStats::new(stats_df)?;

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

        Ok(Self {
            n_rows,
            n_cols,
            column_stats,
            column_map,
        })
    }
}

/// Struct to hold an individual feature's (column) descriptive analysis results.
pub struct FeatureStats(DataFrame);

impl FeatureStats {
    /// Defines the Feature stats schema.
    pub fn schema() -> Schema {
        Schema::from_iter(vec![
            Field::new("column_name", DataType::String),
            Field::new("min", DataType::Float64),
            Field::new("max", DataType::Float64),
            Field::new("mean", DataType::Float64),
            Field::new("median", DataType::Float64),
            Field::new("std_dev", DataType::Float64),
            Field::new("q1", DataType::Float64),
            Field::new("q3", DataType::Float64),
            Field::new("iqr", DataType::Float64),
            Field::new("skewness_bias", DataType::Float64),
            Field::new("skewness_raw", DataType::Float64),
            Field::new("kurtosis", DataType::Float64),
            Field::new("count", DataType::UInt64),
        ])
    }

    /// Constructor for the FeatureStats struct.
    ///
    /// ### Parameters
    ///
    /// - `df`: Dataframe to convert to a FeatureStats struct.
    pub fn new(df: DataFrame) -> Result<Self, DescriptiveError> {
        if df.schema() != Self::schema() {
            return Err(DescriptiveError::Schema());
        }

        Ok(Self(df))
    }

    /// Gets the count value from the first row.
    ///
    /// ### Returns
    ///
    /// - `Result<u64, DescriptiveError>`: The extracted count value.
    pub fn get_count(&self, column_map: IndexMap<String, usize>) -> Result<u64, DescriptiveError> {
        let value = self
            .0
            .column("count")
            .map_err(|_| DescriptiveError::InvalidCol("count".to_owned()))?
            .get(column_map.get("count").unwrap().to_owned())?;
    }
}
