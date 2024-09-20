//! Missing Value Visualizations Module
//!
//! This module handles the generation of the visualiations for the missing value analysis.

use super::{
    create_basic_chart_template, create_drawing_backend, fill_background, LABEL_STYLE,
    PLOT_CAPTION_FONT, PLOT_HEIGHT, PLOT_MARGIN, PLOT_WIDTH, X_LABEL_AREA_SIZE, Y_LABEL_AREA_SIZE,
};
use crate::data::missing_values::MissingValueAnalysis;
use plotters::prelude::*;
use polars::{lazy::dsl::*, prelude::*};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MissingValuesPlotError {
    /// Occurs during failure to build the missing matrix.
    #[error("Error building the missing matrix: {0}")]
    BuildMissingMatrixError(String),

    /// Occurs during failure to build the missingness correlation matrix.
    #[error("Error building the missing matrix: {0}")]
    BuildMissingnessMatrixError(String),

    /// Occurs during failure to draw a chart.
    #[error("Error building the plot: {0}")]
    PlotDrawingError(String),
}

pub fn build_all_visualizations(
    df: &DataFrame,
    missing_values_analysis: &MissingValueAnalysis,
    plot_dir: &PathBuf,
) -> Result<HashMap<String, PathBuf>, MissingValuesPlotError> {
    let missing_data_heatmap = build_missing_data_heatmap(df, missing_values_analysis, plot_dir)?;
    let missingness_correlation_heatmap =
        build_missingness_correlation_heatmap(df, missing_values_analysis, plot_dir)?;
    let missing_value_plot_map =
        HashMap::from([missing_data_heatmap, missingness_correlation_heatmap]);
    return Ok(missing_value_plot_map);
}

/// Creates a heatmap visualization of missing values in the dataset.
///
/// ### Parameters
///
/// - `df`: Reference to the dataset `LazyFrame`.
/// - `missing_values_analysis`: Reference to the `MissingValueAnalysis` struct for the dataset.
/// - `plot_dir`: Reference to the `PathBuf` where the plot should be saved.
///
/// ### Returns
///
/// - `Result<(String, PathBuf), MissingValuesPlotError>`: Result containing a tuple with the plot
/// title (String) and the output file path (PathBuf), or a `MissingValuesPlotError`.
pub fn build_missing_data_heatmap(
    df: &DataFrame,
    missing_values_analysis: &MissingValueAnalysis,
    plot_dir: &PathBuf,
) -> Result<(String, PathBuf), MissingValuesPlotError> {
    let plot_title = "Missing Values Heatmap".to_owned();

    // Prepare the matrix.
    let columns: Vec<&str> = missing_values_analysis
        .column_missing_values
        .keys()
        .map(String::as_str)
        .collect();
    let matrix = build_missing_matrix(df, &columns)?;

    let output_path = plot_dir.join("missing_values_heatmap.png");
    // There's probably a better way to do this.
    let output_path_clone = output_path.clone();

    let root = create_drawing_backend(&output_path_clone, (PLOT_WIDTH, PLOT_HEIGHT));
    fill_background(&root, &WHITE, Some(0.95))
        .map_err(|e| MissingValuesPlotError::PlotDrawingError(e.to_string()))?;

    // Create the chart builder for the heatmap.
    let mut chart = create_basic_chart_template(
        &root,
        &plot_title,
        PLOT_CAPTION_FONT,
        PLOT_MARGIN,
        X_LABEL_AREA_SIZE,
        Y_LABEL_AREA_SIZE,
        (0..matrix[0].len(), 0..matrix.len()),
    )
    .map_err(|e| MissingValuesPlotError::PlotDrawingError(e.to_string()))?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Columns")
        .x_label_style(LABEL_STYLE)
        .y_desc("Rows")
        .y_label_style(LABEL_STYLE)
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
            MissingValuesPlotError::PlotDrawingError(format!(
                "Error configuring chart mesh for missing heatmap: {}",
                e
            ))
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
            MissingValuesPlotError::PlotDrawingError(format!(
                "Error drawing series on plot for missing heatmap: {}",
                e
            ))
        })?;

    Ok((plot_title.clone(), output_path))
}

pub fn build_missingness_correlation_heatmap(
    df: &DataFrame,
    missing_values_analysis: &MissingValueAnalysis,
    plot_dir: &PathBuf,
) -> Result<(String, PathBuf), MissingValuesPlotError> {
    let plot_title = "Missingness Correlation Heatmap".to_owned();

    // Sample the data and prepare the correlation matrix.
    let columns: Vec<&str> = missing_values_analysis
        .column_missing_values
        .keys()
        .map(String::as_str)
        .collect();
    let correlation_matrix = build_missingness_matrix(df, &columns)?;

    let output_path = plot_dir.join("missingness_correlation_heatmap.png");
    // There's probably a better way to do this.
    let output_path_clone = output_path.clone();

    let root = create_drawing_backend(&output_path_clone, (PLOT_WIDTH, PLOT_HEIGHT));
    fill_background(&root, &WHITE, Some(0.95))
        .map_err(|e| MissingValuesPlotError::PlotDrawingError(e.to_string()))?;

    let mut chart = create_basic_chart_template(
        &root,
        &plot_title,
        PLOT_CAPTION_FONT,
        PLOT_MARGIN,
        X_LABEL_AREA_SIZE,
        Y_LABEL_AREA_SIZE,
        (0..columns.len(), 0..columns.len()),
    )
    .map_err(|e| MissingValuesPlotError::PlotDrawingError(e.to_string()))?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_desc("Columns")
        .x_label_style(LABEL_STYLE)
        .y_desc("Rows")
        .y_label_style(LABEL_STYLE)
        .x_labels(columns.len())
        .x_label_formatter(&|x| {
            if *x < columns.len() {
                columns[*x].to_owned()
            } else {
                "".to_owned()
            }
        })
        .y_labels(columns.len())
        .y_label_formatter(&|y| {
            if *y < columns.len() {
                columns[*y].to_owned()
            } else {
                "".to_owned()
            }
        })
        .draw()
        .map_err(|e| {
            MissingValuesPlotError::PlotDrawingError(format!(
                "Error configruing chart mesh for missingness correlation heatmap: {}",
                e
            ))
        })?;

    chart
        .draw_series(correlation_matrix.iter().enumerate().flat_map(|(r, row)| {
            row.iter().enumerate().map(move |(x, &correlation)| {
                let color = RGBColor(
                    (255.0 * (1.0 - correlation.abs())) as u8,
                    (255.0 * (1.0 - correlation.abs())) as u8,
                    255,
                );
                Rectangle::new([(x, r), (x + 1, r + 1)], color.filled())
            })
        }))
        .map_err(|e| {
            MissingValuesPlotError::PlotDrawingError(format!(
                "Error drawing series on plot for missingness correlation heatmap: {}",
                e
            ))
        })?;

    Ok((plot_title.clone(), output_path))
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

fn build_missingness_matrix(
    df: &DataFrame,
    columns: &[&str],
) -> Result<Vec<Vec<f64>>, MissingValuesPlotError> {
    let mut correlation_matrix = vec![vec![0.0; columns.len()]; columns.len()];

    for (i, col1) in columns.iter().enumerate() {
        for (j, col2) in columns.iter().enumerate() {
            if i == j {
                correlation_matrix[i][j] = 1.0;
                continue;
            }

            let is_missing1: Vec<bool> = df
                .column(col1)
                .map_err(|e| {
                    MissingValuesPlotError::BuildMissingnessMatrixError(format!(
                        "Error accessing column {}: {}",
                        col1, e
                    ))
                })?
                .is_null()
                .into_iter()
                .map(|opt| opt.unwrap_or(false)) // Convert Option<bool> to bool
                .collect();

            let is_missing2: Vec<bool> = df
                .column(col2)
                .map_err(|e| {
                    MissingValuesPlotError::BuildMissingnessMatrixError(format!(
                        "Error accessing column {}: {}",
                        col1, e
                    ))
                })?
                .is_null()
                .into_iter()
                .map(|opt| opt.unwrap_or(false))
                .collect();

            let correlation = calculate_pearson_coefficient(&is_missing1, &is_missing2);
            correlation_matrix[i][j] = correlation;
            correlation_matrix[j][i] = correlation;
        }
    }

    Ok(correlation_matrix)
}

fn calculate_pearson_coefficient(x: &[bool], y: &[bool]) -> f64 {
    let n = x.len() as f64;
    let mean_x = x.iter().map(|&b| b as u8 as f64).sum::<f64>() / n;
    let mean_y = y.iter().map(|&b| b as u8 as f64).sum::<f64>() / n;

    let numerator: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(&a, &b)| ((a as u8 as f64 - mean_x) * (b as u8 as f64 - mean_y)))
        .sum();

    let denominator_x: f64 = x
        .iter()
        .map(|&a| (a as u8 as f64 - mean_x).powi(2))
        .sum::<f64>()
        .sqrt();

    let denominator_y: f64 = y
        .iter()
        .map(|&b| (b as u8 as f64 - mean_y).powi(2))
        .sum::<f64>()
        .sqrt();

    if denominator_x == 0.0 || denominator_y == 0.0 {
        0.0
    } else {
        numerator / (denominator_x * denominator_y)
    }
}
