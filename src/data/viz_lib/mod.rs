use plotters::{
    backend::{BitMapBackend, DrawingBackend},
    chart::{ChartBuilder, ChartContext},
    coord::{cartesian::Cartesian2d, ranged1d::AsRangedCoord, Shift},
    drawing::{DrawingArea, IntoDrawingArea},
    style::Color,
};
use std::path::PathBuf;
use thiserror::Error;

pub mod missing_value_viz;

pub const PLOT_WIDTH: u32 = 1200;
pub const PLOT_HEIGHT: u32 = 800;
pub const _FONT: &str = "sans-serif";
pub const PLOT_CAPTION_FONT: (&str, u32) = (_FONT, 35);
pub const PLOT_MARGIN: u32 = 10;
pub const X_LABEL_AREA_SIZE: u32 = 50;
pub const Y_LABEL_AREA_SIZE: u32 = 80;
pub const LABEL_STYLE: (&str, u32) = (_FONT, 16);

#[derive(Error, Debug)]
pub enum DrawingError {
    /// Error filling BitMap Backend.
    #[error("Error filling background color: {0}")]
    FillBackgroundError(String),

    /// Error creating basic chart template.
    #[error("Error creating chart template: {0}")]
    ChartTemplateError(String),
}

pub fn create_drawing_backend(
    path: &PathBuf,
    dimensions: (u32, u32),
) -> DrawingArea<BitMapBackend, Shift> {
    let root = BitMapBackend::new(path, dimensions).into_drawing_area();
    return root;
}

pub fn fill_background<T>(
    root: &DrawingArea<BitMapBackend, Shift>,
    color: &T,
    mix_value: Option<f64>,
) -> Result<(), DrawingError>
where
    T: Color,
{
    match mix_value {
        Some(val) => root
            .fill(&color.mix(val))
            .map_err(|e| DrawingError::FillBackgroundError(e.to_string())),
        None => root
            .fill(&color)
            .map_err(|e| DrawingError::FillBackgroundError(e.to_string())),
    }
}

pub fn create_basic_chart_template<'a, X, Y, DB>(
    root: &'a DrawingArea<DB, Shift>,
    caption: &'a str,
    font_style: (&'a str, u32),
    margin: u32,
    x_label_area_size: u32,
    y_label_area_size: u32,
    cartesian_dimensions: (X, Y),
) -> Result<ChartContext<'a, DB, Cartesian2d<X::CoordDescType, Y::CoordDescType>>, DrawingError>
where
    X: AsRangedCoord,
    Y: AsRangedCoord,
    DB: DrawingBackend + 'a,
{
    let chart = ChartBuilder::on(root)
        .caption(caption, font_style)
        .margin(margin)
        .x_label_area_size(x_label_area_size)
        .y_label_area_size(y_label_area_size)
        .build_cartesian_2d(cartesian_dimensions.0, cartesian_dimensions.1)
        .map_err(|e| DrawingError::ChartTemplateError(e.to_string()))?;

    Ok(chart)
}
