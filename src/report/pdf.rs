//! # PDF Module
//!
//! Handles the base implementation of generating a comprehensive PDF report with the exploratory
//! analysis findings.

use indexmap::IndexMap;
use pdfium_render::prelude::*;
use polars::datatypes::DataType;
use std::path::PathBuf;
use thiserror::Error;

/// The default paper size.
pub const PAPER_SIZE: PdfPagePaperStandardSize = PdfPagePaperStandardSize::A4;
/// The default font.
pub const FONT: PdfFontBuiltin = PdfFontBuiltin::Helvetica;
/// The default bold font.
pub const BOLD_FONT: PdfFontBuiltin = PdfFontBuiltin::HelveticaBold;
/// The default italic font.
pub const ITALIC_FONT: PdfFontBuiltin = PdfFontBuiltin::HelveticaOblique;
/// Section header font size.
pub const SECTION_HEADER_FONT_SIZE: f32 = 24.0;
/// Normal text font size.
pub const FONT_SIZE: f32 = 16.0;

/// The error types for the pdf modules.
#[derive(Error, Debug)]
pub enum PdfError {
    /// Occurs on a Pdfium library error.
    #[error("Pdf error: {0}")]
    Pdfium(#[from] pdfium_render::error::PdfiumError),
}

/// Struct that keeps track of the current page position and number. Allows for manual page
/// management, page break handling, and flow content across multiple pages.
pub struct PageManager<'a> {
    /// The PDF document instance.
    document: PdfDocument<'a>,
    /// The current page in the document.
    current_page: u32,
    /// The current y position on the current page to keep track of when a new page should be created.
    y_position: f32,
    /// The height of the current page.
    page_height: f32,
    /// The width of the current page.
    page_width: f32,
    /// The regular font.
    font: PdfFontToken,
    /// The bold font.
    bold_font: PdfFontToken,
    /// The italic font.
    italic_font: PdfFontToken,
}

impl<'a> PageManager<'a> {
    /// Constructor for the PageManager struct.
    ///
    /// ### Parameters
    ///
    /// - `pdfium`: Reference to a Pdfium struct.
    ///
    /// ### Returns
    ///
    /// - `PageManager`: The new PageManager.
    ///
    pub fn new(pdfium: &'a Pdfium) -> Result<Self, PdfError> {
        let mut document = pdfium.create_new_pdf()?;
        let font = document.fonts_mut().new_built_in(FONT);
        let bold_font = document.fonts_mut().new_built_in(BOLD_FONT);
        let italic_font = document.fonts_mut().new_built_in(ITALIC_FONT);
        Ok(PageManager {
            document,
            current_page: 0,
            y_position: 0.0,
            page_height: PAPER_SIZE.height().value,
            page_width: PAPER_SIZE.width().value,
            font,
            bold_font,
            italic_font,
        })
    }

    /// Create the report title page.
    ///
    /// ### Parameters
    ///
    /// - `data_title`: Title of the dataset.
    ///
    /// ### Returns
    ///
    /// - `Result<(), PdfError>`: Unit type or a propagated PdfError.
    ///
    pub fn create_title_page(&mut self, data_title: &str) -> Result<(), PdfError> {
        let mut page = self
            .document
            .pages_mut()
            .create_page_at_start(PdfPagePaperSize::new_portrait(PAPER_SIZE))?;

        // Add main document title.
        let mut title_object = PdfPageTextObject::new(
            &self.document,
            "Exploratory Data Analysis Report",
            self.bold_font,
            PdfPoints::new(48.0),
        )?;
        title_object.set_fill_color(PdfColor::new(0, 0, 0, 255))?;
        title_object.translate(
            PdfPoints::new(self.page_width * 0.1),
            PdfPoints::new(self.page_height * 0.9),
        )?;
        page.objects_mut().add_text_object(title_object)?;

        // Add dataset subtitle.
        let mut dataset_title_object = PdfPageTextObject::new(
            &self.document,
            &format!("Dataset: {}", data_title),
            self.font,
            PdfPoints::new(18.0),
        )?;
        dataset_title_object.set_fill_color(PdfColor::new(0, 0, 0, 255))?;
        dataset_title_object.translate(
            PdfPoints::new(self.page_width * 0.1),
            PdfPoints::new(self.page_height * 0.85),
        )?;
        page.objects_mut().add_text_object(dataset_title_object)?;

        // Add date.
        let date = chrono::Local::now().format("%m-%d-%Y").to_string();
        let mut date_object = PdfPageTextObject::new(
            &self.document,
            format!("Generated on: {}", date),
            self.font,
            PdfPoints::new(12.0),
        )?;
        date_object.set_fill_color(PdfColor::new(0, 0, 0, 255))?;
        date_object.translate(
            PdfPoints::new(self.page_width * 0.1),
            PdfPoints::new(self.page_height * 0.75),
        )?;
        page.objects_mut().add_text_object(date_object)?;

        self.current_page += 1;
        self.y_position = 0.0;

        Ok(())
    }

    ///
    pub fn create_data_types_page(
        &mut self,
        column_types: &IndexMap<String, DataType>,
    ) -> Result<(), PdfError> {
        let mut title_object = PdfPageTextObject::new(
            &self.document,
            "Data Types Overview",
            self.bold_font,
            PdfPoints::new(SECTION_HEADER_FONT_SIZE),
        )?;
    }

    /// Saves the document to disk.
    ///
    /// ### Parameters
    ///
    /// - `path`: Path to save the file.
    ///
    /// ### Returns
    ///
    /// - `Result<(), PdfError>`: Unit type of a propagated PdfError.
    ///
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), PdfError> {
        self.document.save_to_file(path)?;
        Ok(())
    }

    /// Helper function to create and position a text object on the page.
    ///
    /// ### Parameters
    ///
    /// - `text`: The text content.
    /// - `font`: The font to use.
    /// - `font_size`: The font size.
    /// - `x`: The x-coordinate (in points).
    /// - `y`: The y-coordinate (in points).
    /// - `color`: Optional color (defaults to black if None).
    ///
    /// ### Returns
    ///
    /// - `Result<PdfPageTextObject, PdfError>`: The created and positioned text object or a PDF
    /// error.
    ///
    fn create_text_object(
        &self,
        text: &str,
        font: PdfFontToken,
        font_size: f32,
        x: f32,
        y: f32,
        color: Option<PdfColor>,
    ) -> Result<PdfPageTextObject, PdfError> {
        let mut text_object =
            PdfPageTextObject::new(&self.document, text, font, PdfPoints::new(font_size))?;
        text_object.set_fill_color(color.unwrap_or(PdfColor::new(0, 0, 0, 255)))?;
        text_object.translate(PdfPoints::new(x), PdfPoints::new(y))?;
        Ok(text_object)
    }
}
