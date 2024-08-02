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
pub const FONT: PdfFontBuiltin = PdfFontBuiltin::TimesRoman;
/// The default bold font.
pub const BOLD_FONT: PdfFontBuiltin = PdfFontBuiltin::TimesBold;
/// The default italic font.
pub const ITALIC_FONT: PdfFontBuiltin = PdfFontBuiltin::TimesItalic;
/// Section header font size.
pub const SECTION_HEADER_FONT_SIZE: f32 = 24.0;
/// Normal text font size.
pub const FONT_SIZE: f32 = 16.0;
/// Bottom page margin.
pub const BOTTOM_MARGIN: f32 = 0.1;

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
    /// Section page tracker for table of contents.
    section_page_map: IndexMap<String, u32>,
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
    pub fn new(pdfium: &'a Pdfium) -> Result<Self, PdfError> {
        let mut document = pdfium.create_new_pdf()?;
        let font = document.fonts_mut().new_built_in(FONT);
        let bold_font = document.fonts_mut().new_built_in(BOLD_FONT);
        let italic_font = document.fonts_mut().new_built_in(ITALIC_FONT);
        Ok(PageManager {
            document,
            current_page: 0,
            page_height: PAPER_SIZE.height().value,
            page_width: PAPER_SIZE.width().value,
            font,
            bold_font,
            italic_font,
            section_page_map: IndexMap::new(),
        })
    }

    /// Generates the final report.
    ///
    /// ### Parameters
    ///
    /// - `data_title`: Title of the dataset.
    /// - `column_types`: Index map of the columns and their corresponding data types.
    pub fn generate_report(
        &mut self,
        data_title: &str,
        column_types: &IndexMap<String, DataType>,
    ) -> Result<(), PdfError> {
        self.create_title_page(data_title)?;
        self.create_data_types_page(column_types)?;
        self.create_table_of_contents()?;
        Ok(())
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
    pub fn create_title_page(&mut self, data_title: &str) -> Result<(), PdfError> {
        self.new_page()?;

        // Add main document title.
        self.add_text("Exploratory Data", self.bold_font, 48.0, 0.1, 0.9, None)?;
        self.add_text("Analysis Report", self.bold_font, 48.0, 0.1, 0.83, None)?;
        //
        // Add a horizontal line.
        self.add_line(0.1, 0.80, 0.9, 0.8, 2.0)?;

        // Add dataset subtitle.
        self.add_text(
            &format!("Dataset: {}", data_title),
            self.font,
            18.0,
            0.1,
            0.75,
            None,
        )?;

        // Add date.
        let date = chrono::Local::now().format("%B %d, %Y").to_string();
        self.add_text(
            &format!("Generated on: {}", date),
            self.font,
            12.0,
            0.1,
            0.70,
            None,
        )?;

        Ok(())
    }

    /// Creates the table of contents page(s).
    ///
    /// ### Returns
    ///
    /// - `Result<u32, PdfError>`: Unit type or a propagated PdfError.
    pub fn create_table_of_contents(&mut self) -> Result<u32, PdfError> {
        let start_page = 1;
        self.insert_page_at(start_page)?;
        let mut pages_added = 1;

        self.add_text(
            "Table of Contents",
            self.bold_font,
            SECTION_HEADER_FONT_SIZE,
            0.1,
            0.9,
            None,
        )?;

        let mut y_fraction = 0.85;
        let line_height_fraction = FONT_SIZE / self.page_height;

        let sections: Vec<(String, u32)> = self
            .section_page_map
            .iter()
            .map(|(name, &page)| (name.clone(), page))
            .collect();

        for (section_name, page_number) in sections {
            if self.need_new_page(y_fraction, line_height_fraction) {
                self.insert_page_at(start_page + pages_added)?;
                pages_added += 1;
                y_fraction = 0.9;
            }

            let text = format!(
                "{} ........................... {}",
                section_name,
                page_number + pages_added as u32
            );
            self.add_text(&text, self.font, FONT_SIZE, 0.1, y_fraction, None)?;
            y_fraction -= line_height_fraction;
        }

        for page_number in self.section_page_map.values_mut() {
            *page_number += pages_added as u32;
        }

        Ok(pages_added as u32)
    }

    /// Creates the column type overview page.
    ///
    /// ### Parameters
    ///
    /// - `column_types`: The index map of the column names and corresponding data types.
    ///
    /// ### Returns
    ///
    /// - `Result<(), PdfError>`: Unit type or the propagated PdfError.
    pub fn create_data_types_page(
        &mut self,
        column_types: &IndexMap<String, DataType>,
    ) -> Result<(), PdfError> {
        self.new_page()?;

        self.section_page_map
            .insert("Data Types Overview".to_owned(), self.current_page - 1);
        self.add_text(
            "Data Types Overview",
            self.bold_font,
            SECTION_HEADER_FONT_SIZE,
            0.1,
            0.9,
            None,
        )?;

        let mut y_fraction = 0.85;
        let line_height_fraction = FONT_SIZE / self.page_height;

        for (column_name, data_type) in column_types {
            if self.need_new_page(y_fraction, line_height_fraction) {
                self.new_page()?;
                y_fraction = 0.9;
            }

            let text = format!("{}: {}", column_name, data_type);
            self.add_text(&text, self.font, FONT_SIZE, 0.1, y_fraction, None)?;

            y_fraction -= line_height_fraction;
        }

        Ok(())
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
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), PdfError> {
        self.document.save_to_file(path)?;
        Ok(())
    }

    /// Helper function to add text to a page.
    ///
    /// ### Parameters
    ///
    /// - `text`: The text content.
    /// - `font`: The font to use.
    /// - `font_size`: The font size.
    /// - `x_fraction`: The x fraction to place the text.
    /// - `y_fraction`: The y fraction to place the text.
    /// - `color`: Optional color (defaults to black if None).
    ///
    /// ### Returns
    ///
    /// - `Result<PdfPageTextObject, PdfError>`: The created and positioned text object or a PDF
    /// error.
    fn add_text(
        &mut self,
        text: &str,
        font: PdfFontToken,
        font_size: f32,
        x_fraction: f32,
        y_fraction: f32,
        color: Option<PdfColor>,
    ) -> Result<(), PdfError> {
        let mut text_object =
            PdfPageTextObject::new(&self.document, text, font, PdfPoints::new(font_size))?;
        text_object.set_fill_color(color.unwrap_or(PdfColor::new(0, 0, 0, 255)))?;
        text_object.translate(
            PdfPoints::new(self.page_width * x_fraction),
            PdfPoints::new(self.page_height * y_fraction),
        )?;
        let mut current_page = self.document.pages().get(self.current_page as u16).unwrap();
        current_page.objects_mut().add_text_object(text_object)?;
        Ok(())
    }

    /// Creates a new page at the end of the document.
    fn new_page(&mut self) -> Result<(), PdfError> {
        self.document
            .pages_mut()
            .create_page_at_end(PdfPagePaperSize::new_portrait(PAPER_SIZE))?;
        self.current_page = self.document.pages().len() as u32 - 1;
        Ok(())
    }

    // Adds a horizontal line.
    fn add_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, width: f32) -> Result<(), PdfError> {
        let mut path = PdfPagePathObject::new(
            &self.document,
            PdfPoints::new(self.page_width * x1),
            PdfPoints::new(self.page_height * y1),
            Some(PdfColor::new(0, 0, 0, 255)),
            Some(PdfPoints::new(width)),
            None,
        )?;

        path.line_to(PdfPoints::new(self.page_width * x2), PdfPoints::new(self.page_height * y2))?;

        let mut current_page = self.document.pages().get(self.current_page as u16).unwrap();
        current_page.objects_mut().add_path_object(path)?;
        Ok(())
    }

    /// Creates a new page at a specified index in the document.
    fn insert_page_at(&mut self, index: u16) -> Result<(), PdfError> {
        self.document
            .pages_mut()
            .create_page_at_index(PdfPagePaperSize::new_portrait(PAPER_SIZE), index)?;
        self.current_page = index as u32;
        Ok(())
    }

    /// Based on the Y coordinate page fraction and the content height fraction determine whether a
    /// new page is needed.
    fn need_new_page(&self, y_fraction: f32, content_height_fraction: f32) -> bool {
        y_fraction - content_height_fraction < BOTTOM_MARGIN
    }
}
