//! Missing Value Visualizations Module
//!
//! This module handles the generation of the visualiations for the missing value analysis.

use super::{SampleModeEnum, SAMPLE_LIMIT, SAMPLE_MODE, SAMPLE_RATIO};
use crate::data::missing_values::MissingValueAnalysis;
use plotters::prelude::*;
use polars::{lazy::dsl::*, prelude::*};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MissingValuesPlotError {
    /// Occurs when sampling the lazy frame fails.
    #[error("Error sampling the dataframe: {0}")]
    DataFrameSamplingError(String),

    /// Occurs during failure to build the missing matrix.
    #[error("Error building the missing matrix: {0}")]
    BuildMissingMatrixError(String),

    /// Occurs during failure to draw a chart.
    #[error("Error building the plot: {0}")]
    PlotDrawingError(String),
}

/// Creates a heatmap visualiation of missing values in the dataset.
///
/// ### Parameters
///
/// - `lazy_df`: Reference to the dataset `LazyFrame`.
/// - `missing_values_analysis`: Reference to the `MissingValueAnalysis` struct for the dataset.
/// - `plot_dir`: Reference to the `PathBuf` where the plot should be saved.
///
/// ### Returns
///
/// - `Result<(String, PathBuf), MissingValuesPlotError>`: Result containing a tuple with the plot
/// title (String) and the output file path (PathBuf), or a `MissingValuesPlotError`.
pub fn build_missing_data_matrix(
    lazy_df: &LazyFrame,
    missing_values_analysis: &MissingValueAnalysis,
    plot_dir: &PathBuf,
) -> Result<(String, PathBuf), MissingValuesPlotError> {
    let plot_title = "Missing Values Heatmap".to_owned();

    // Sample the data and prepare the matrix.
    let sampled_df = sample_dataframe(lazy_df)?;
    let columns: Vec<&str> = missing_values_analysis
        .column_missing_values
        .keys()
        .map(String::as_str)
        .collect();
    let matrix = build_missing_matrix(&sampled_df, &columns)?;

    let output_path = plot_dir.join("missing_values_heatmap.png");
    // There's probably a better way to do this.
    let output_path_clone = output_path.clone();

    // Create a new drawing backend that is a 1000x600 pixel image.
    let root = BitMapBackend::new(&output_path_clone, (1200, 800)).into_drawing_area();

    // Fill the background of the plot with white.
    root.fill(&WHITE.mix(0.95)).map_err(|e| {
        MissingValuesPlotError::PlotDrawingError(format!(
            "Error filling the plot background color to white: {}",
            e
        ))
    })?;

    // Split the drawing area into two parts: top for the heatmap, bottom for the legend.
    // let (top, _bottom) = root.split_vertically(750);

    // Create the chart builder for the heatmap.
    let mut chart = ChartBuilder::on(&root)
        .caption(&plot_title, ("sans-serif", 35))
        .margin(5)
        .x_label_area_size(50)
        .y_label_area_size(80)
        .build_cartesian_2d(0..matrix[0].len(), 0..matrix.len())
        .map_err(|e| {
            MissingValuesPlotError::PlotDrawingError(format!(
                "Error building cartesian 2d coordinate system: {}",
                e
            ))
        })?;
    // Configure the mesh (grid) for the chart.
    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Columns")
        .y_desc("Rows")
        .x_labels(columns.len())
        .x_label_formatter(&|x| {
            if *x < columns.len() {
                columns[*x].to_owned()
            } else {
                "".to_owned()
            }
        })
        .y_labels(matrix.len().min(20))
        .y_label_formatter(&|y| format!("Row {}", y))
        .draw()
        .map_err(|e| {
            MissingValuesPlotError::PlotDrawingError(format!("Error configuring chart mesh: {}", e))
        })?;

    // Draw the heatmap.
    chart
        .draw_series(matrix.iter().enumerate().flat_map(|(y, row)| {
            row.iter().enumerate().map(move |(x, &is_missing)| {
                let color = if is_missing { &RED } else { &WHITE };
                Rectangle::new([(x, y), (x + 1, y + 1)], color.filled())
            })
        }))
        .map_err(|e| {
            MissingValuesPlotError::PlotDrawingError(format!("Error drawing series on plot: {}", e))
        })?;

    // Create the chart builder for the legend.
    // let mut legend = ChartBuilder::on(&bottom)
    //     .margin(5)
    //     .x_label_area_size(30)
    //     .y_label_area_size(30)
    //     .build_cartesian_2d(0..1, 0..1)
    //     .map_err(|e| {
    //         MissingValuesPlotError::PlotDrawingError(format!("Error building chart legend: {}", e))
    //     })?;
    //
    // // Draw the legend.
    // legend
    //     .draw_series(std::iter::once(Rectangle::new(
    //         [(0, 0), (20, 20)],
    //         RED.filled(),
    //     )))
    //     .map_err(|e| {
    //         MissingValuesPlotError::PlotDrawingError(format!(
    //             "Error drawing missing value legend value: {}",
    //             e
    //         ))
    //     })?
    //     .label("Missing")
    //     .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));
    //
    // // Configure and draw the legend label(s).
    // legend
    //     .configure_series_labels()
    //     .background_style(WHITE.mix(0.8))
    //     .border_style(BLACK)
    //     .position(SeriesLabelPosition::UpperRight)
    //     .draw()
    //     .map_err(|e| {
    //         MissingValuesPlotError::PlotDrawingError(format!("Error configuring legend: {}", e))
    //     })?;

    Ok((plot_title, output_path))
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
fn sample_dataframe(lazy_df: &LazyFrame) -> Result<DataFrame, MissingValuesPlotError> {
    match SAMPLE_MODE {
        SampleModeEnum::Limit => {
            let df = lazy_df
                .clone()
                .select([all().sample_n(lit(SAMPLE_LIMIT), false, false, None)])
                .collect()
                .map_err(|e| {
                    MissingValuesPlotError::DataFrameSamplingError(format!(
                        "Couldn't collect limit ({}) sampled dataframe: {}",
                        SAMPLE_LIMIT, e
                    ))
                })?;
            Ok(df)
        }
        SampleModeEnum::Ratio => {
            let df = lazy_df
                .clone()
                .select([all().sample_frac(lit(SAMPLE_RATIO), false, false, None)])
                .collect()
                .map_err(|e| {
                    MissingValuesPlotError::DataFrameSamplingError(format!(
                        "Couldn't collect ratio ({}) sampled dataframe: {}",
                        SAMPLE_RATIO, e
                    ))
                })?;
            Ok(df)
        }
        SampleModeEnum::Full => {
            let df = lazy_df.clone().collect().map_err(|e| {
                MissingValuesPlotError::DataFrameSamplingError(format!(
                    "Couldn't collect lazy frame: {}",
                    e
                ))
            })?;
            Ok(df)
        }
    }
}

/// Constructs a matrix representing missing values in the dataset.
///
/// ### Parameters
///
/// - `df`: Reference to the dataset `DataFrame`.
/// - `columns`: Reference to the vector of column names that contain missing values.
///
/// ### Returns
///
/// - `Result<Vec<Vec<bool>>, MissingValuesPlotError>`: A Result containing a 2D vector
/// of booleans representing missing (true) or present (false) values, or a MissingValuesPlotError
/// if an error occurs during matrix construction.
fn build_missing_matrix(
    df: &DataFrame,
    columns: &Vec<&str>,
) -> Result<Vec<Vec<bool>>, MissingValuesPlotError> {
    let mut matrix = Vec::with_capacity(df.height());
    for col_name in columns {
        let column = df.column(col_name).map_err(|e| {
            MissingValuesPlotError::BuildMissingMatrixError(format!("here1: {}", e))
        })?;

        let mut row_index = 0;

        for chunk in column.chunks() {
            let chunk_len = chunk.len();

            if matrix.len() < row_index + chunk_len {
                matrix.extend((0..chunk_len).map(|i| vec![chunk.is_null(i)]));
            } else {
                for i in 0..chunk_len {
                    if let Some(row) = matrix.get_mut(row_index + i) {
                        row.push(chunk.is_null(i));
                    }
                }
            }
            row_index += chunk_len;
        }
    }
    Ok(matrix)
}
