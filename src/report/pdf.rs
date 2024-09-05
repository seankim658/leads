//! # PDF Module
//!
//! Handles the base implementation of generating a comprehensive PDF report with the exploratory
//! analysis findings.

use crate::prelude::{DataInfo, DescriptiveAnalysis, LeadsError, MissingValueAnalysis};
use indexmap::IndexMap;
use pdfium_render::prelude::*;
use polars::datatypes::DataType;
use std::path::PathBuf;
use thiserror::Error;

use super::glossary::{get_data_type_category, Glossary};

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
/// Sub-header for feature names.
pub const FEATURE_HEADER_FONT_SIZE: f32 = 14.0;
/// Normal text font size.
pub const FONT_SIZE: f32 = 12.0;
/// Bottom page margin.
pub const BOTTOM_MARGIN: f32 = 0.1;
/// Padding between normal lines of text.
pub const LINE_HEIGHT_PADDING: f32 = 0.005;

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
    /// The height of the current page in points.
    page_height: f32,
    /// The width of the current page in points.
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
    /// - `data_info`: The dataset information.
    pub fn generate_report(&mut self, data_info: &DataInfo) -> Result<(), LeadsError> {
        self.create_title_page(&data_info.data_title)?;
        self.create_data_types_page(&data_info.column_types)?;
        self.create_descriptive_analysis_page(&data_info.descriptive_analysis)?;
        self.create_missing_values_page(&data_info.missing_value_analysis)?;
        self.create_glossary_page()?;
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
    pub fn create_title_page(&mut self, data_title: &str) -> Result<(), LeadsError> {
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
            24.0,
            0.1,
            0.75,
            None,
        )?;

        // Add a brief description of the report.
        self.add_text(
            "This report provides a comprehensive exploratory analysis of the dataset,",
            self.font,
            14.0,
            0.1,
            0.65,
            None,
        )?;
        self.add_text(
            "including statistical summaries, outliers, visualizations, and key insights.",
            self.font,
            14.0,
            0.1,
            0.62,
            None,
        )?;

        // Add date.
        let date = chrono::Local::now().format("%B %d, %Y").to_string();
        self.add_text(
            &format!("Generated on: {}", date),
            self.font,
            12.0,
            0.1,
            0.2,
            None,
        )?;

        // Add crate version.
        let version = env!("CARGO_PKG_VERSION");
        self.add_text(
            &format!("LEADS version: {}", version),
            self.font,
            12.0,
            0.1,
            0.17,
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
        let line_height_fraction = FONT_SIZE / self.page_height + LINE_HEIGHT_PADDING;

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

            self.add_text(&section_name, self.font, FONT_SIZE, 0.1, y_fraction, None)?;

            let page_num_text = format!("{}", page_number + pages_added as u32);
            self.add_text(&page_num_text, self.font, FONT_SIZE, 0.9, y_fraction, None)?;

            let section_width = self.get_text_width(&section_name, self.font, FONT_SIZE)?;
            let page_num_width = self.get_text_width(&page_num_text, self.font, FONT_SIZE)?;

            let start_x = 0.1 + section_width + 0.01;
            let end_x = 0.9 - page_num_width - 0.01;

            self.add_dotted_line(start_x, end_x, y_fraction)?;

            y_fraction -= line_height_fraction;
        }

        for i in 0..pages_added {
            let page_num = to_roman_numeral((i + 1).into());
            let mut page = self
                .document
                .pages()
                .get(start_page as u16 + i as u16)
                .unwrap();
            let mut text_object =
                PdfPageTextObject::new(&self.document, &page_num, self.font, PdfPoints::new(10.0))?;
            text_object.set_fill_color(PdfColor::new(0, 0, 0, 255))?;
            text_object.translate(
                PdfPoints::new(self.page_width * 0.95),
                PdfPoints::new(self.page_height * 0.05),
            )?;
            page.objects_mut().add_text_object(text_object)?;
        }

        for page_number in self.section_page_map.values_mut() {
            *page_number += pages_added as u32;
        }
        self.section_page_map
            .insert("Table of Contents".to_owned(), pages_added as u32);
        self.add_page_numbers()?;

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

        let y_start = 0.85;
        let column1_x = 0.1;
        let column2_x = 0.4;
        let column3_x = 0.7;
        let line_height = FONT_SIZE / self.page_height + 2.0 * LINE_HEIGHT_PADDING;

        self.add_text(
            "Feature",
            self.bold_font,
            FONT_SIZE,
            column1_x,
            y_start,
            None,
        )?;
        self.add_text(
            "Data Type",
            self.bold_font,
            FONT_SIZE,
            column2_x,
            y_start,
            None,
        )?;
        self.add_text(
            "Category",
            self.bold_font,
            FONT_SIZE,
            column3_x,
            y_start,
            None,
        )?;

        self.add_line(
            column1_x,
            y_start - 0.5 * line_height,
            0.9,
            y_start - 0.5 * line_height,
            1.0,
        )?;

        let mut y_position = y_start - 2.0 * line_height;
        let mut row_count = 0;

        for (column_name, data_type) in column_types {
            if self.need_new_page(y_position, 3.0 * line_height) {
                self.new_page()?;
                y_position = 0.9;
            }

            if row_count % 2 == 0 {
                self.add_rectangle(
                    column1_x,
                    y_position + line_height,
                    0.9,
                    y_position - line_height,
                    PdfColor::new(240, 240, 240, 255),
                )?;
            }

            self.add_text(
                column_name,
                self.font,
                FONT_SIZE,
                column1_x + 0.01,
                y_position,
                None,
            )?;
            self.add_text(
                &data_type.to_string(),
                self.font,
                FONT_SIZE,
                column2_x,
                y_position,
                None,
            )?;

            let description = get_data_type_category(data_type);
            let wrapped_description =
                self.wrap_text(&description, column3_x, 0.9, self.font, FONT_SIZE);

            for (i, line) in wrapped_description.iter().enumerate() {
                self.add_text(
                    line,
                    self.font,
                    FONT_SIZE,
                    column3_x,
                    y_position - i as f32 * line_height,
                    None,
                )?;
            }

            y_position -= (wrapped_description.len() as f32 + 1.0) * line_height;
            row_count += 1;
        }

        Ok(())
    }

    /// Creates the report pages with the results of the basic descriptie analysis.
    ///
    /// ### Parameters
    ///
    /// - `descriptive_analysis`: The descriptive analysis results.
    ///
    /// ### Returns
    ///
    /// - `Result<() PdfError>`: Unit type or the propogated PdfError.
    pub fn create_descriptive_analysis_page(
        &mut self,
        descriptive_analysis: &DescriptiveAnalysis,
    ) -> Result<(), LeadsError> {
        self.new_page()?;
        self.section_page_map
            .insert("Descriptive Analysis".to_owned(), self.current_page - 1);
        self.add_text(
            "Descriptive Analysis",
            self.bold_font,
            SECTION_HEADER_FONT_SIZE,
            0.1,
            0.9,
            None,
        )?;

        let mut y_fraction = 0.86;
        let line_height_fraction = FONT_SIZE / self.page_height + LINE_HEIGHT_PADDING;
        let feature_line_height_fraction = FEATURE_HEADER_FONT_SIZE / self.page_height;

        self.add_text("Shape:", self.bold_font, FONT_SIZE, 0.1, y_fraction, None)?;
        let shape_txt_width = self.get_text_width("Shape:", self.bold_font, FONT_SIZE)?;
        self.add_text(
            &format!(
                "{} rows, {} columns",
                descriptive_analysis.n_rows, descriptive_analysis.n_cols
            ),
            self.font,
            FONT_SIZE,
            0.1 + shape_txt_width + 0.005,
            y_fraction,
            None,
        )?;
        y_fraction -= 2.0 * line_height_fraction;

        let analysis_values = descriptive_analysis.column_stats.get_analysis_values(
            &descriptive_analysis.feature_indices,
            &descriptive_analysis.column_map,
        )?;

        for feature_stats in analysis_values {
            if self.need_new_page(
                y_fraction,
                feature_line_height_fraction + 7.0 * line_height_fraction,
            ) {
                self.new_page()?;
                y_fraction = 0.9;
            }

            // Add feature sub-header.
            let feature_name = &feature_stats["column_name"];
            self.add_text(
                feature_name,
                self.bold_font,
                FEATURE_HEADER_FONT_SIZE,
                0.1,
                y_fraction,
                None,
            )?;
            y_fraction -= feature_line_height_fraction;

            self.add_line(
                0.1,
                y_fraction + LINE_HEIGHT_PADDING,
                0.9,
                y_fraction + LINE_HEIGHT_PADDING,
                0.5,
            )?;
            y_fraction -= line_height_fraction;

            // Format metrics in two columns.
            let left_column = 0.15;
            let right_column = 0.55;
            let mut counter = 0;

            for (stat_name, stat_value) in feature_stats.iter() {
                if stat_name == "column_name" {
                    continue;
                }

                let x_position = if counter % 2 == 0 {
                    left_column
                } else {
                    right_column
                };

                self.add_text(
                    &format!("{}:", stat_name),
                    self.bold_font,
                    FONT_SIZE,
                    x_position,
                    y_fraction,
                    None,
                )?;

                let value_x = x_position + 0.2;
                self.add_text(&stat_value, self.font, FONT_SIZE, value_x, y_fraction, None)?;

                if counter % 2 == 1 {
                    y_fraction -= line_height_fraction;
                }
                counter += 1;

                if counter % 2 == 0
                    && self.need_new_page(y_fraction - line_height_fraction, line_height_fraction)
                {
                    self.new_page()?;
                    y_fraction = 0.9;
                }
            }

            if counter % 2 == 1 {
                y_fraction -= line_height_fraction;
            }

            y_fraction -= 1.5 * line_height_fraction;
        }

        Ok(())
    }

    /// Create the missing values analysis pages.
    pub fn create_missing_values_page(
        &mut self,
        missing_values_analysis: &MissingValueAnalysis,
    ) -> Result<(), PdfError> {
        self.new_page()?;
        self.section_page_map
            .insert("Missing Values Analysis".to_owned(), self.current_page - 1);

        self.add_text(
            "Missing Values Analysis",
            self.bold_font,
            SECTION_HEADER_FONT_SIZE,
            0.1,
            0.9,
            None,
        )?;

        let mut y_fraction = 0.85;
        let line_height_fraction = FONT_SIZE / self.page_height + (LINE_HEIGHT_PADDING + 0.005);

        // Add table headers (Feature, Missing Count, Missing Percentage).
        self.add_text("Feature", self.bold_font, FONT_SIZE, 0.1, y_fraction, None)?;
        self.add_text(
            "Missing Count",
            self.bold_font,
            FONT_SIZE,
            0.4,
            y_fraction,
            None,
        )?;
        self.add_text(
            "Missing Percentage",
            self.bold_font,
            FONT_SIZE,
            0.7,
            y_fraction,
            None,
        )?;

        // Draw a separator line
        self.add_line(0.1, y_fraction - 0.02, 0.9, y_fraction - 0.02, 1.0)?;

        y_fraction -= 2.0 * line_height_fraction;

        // Iterate over the missing values data and display.
        for (column, (missing_count, missing_percentage)) in
            &missing_values_analysis.column_missing_values
        {
            // Add column name.
            self.add_text(column, self.font, FONT_SIZE, 0.1, y_fraction, None)?;

            // Add missing count.
            self.add_text(
                &format!("{}", missing_count),
                self.font,
                FONT_SIZE,
                0.4,
                y_fraction,
                None,
            )?;

            // Add missing percentage.
            self.add_text(
                &format!("{:.2}%", missing_percentage),
                self.font,
                FONT_SIZE,
                0.7,
                y_fraction,
                None,
            )?;

            // Move to the next line, and add page breaks if necessary.
            y_fraction -= line_height_fraction;
            if self.need_new_page(y_fraction, line_height_fraction) {
                self.new_page()?;
                y_fraction = 0.9;
            }
        }

        Ok(())
    }

    /// Creates the term glossary pages.
    pub fn create_glossary_page(&mut self) -> Result<(), PdfError> {
        self.new_page()?;
        self.section_page_map
            .insert("Glossary".to_owned(), self.current_page - 1);

        self.add_text(
            "Glossary",
            self.bold_font,
            SECTION_HEADER_FONT_SIZE,
            0.1,
            0.9,
            None,
        )?;

        let mut y_fraction = 0.85;
        let term_line_height_fraction = 12.0 / self.page_height + LINE_HEIGHT_PADDING;
        let definition_line_height_fraction = 10.0 / self.page_height + LINE_HEIGHT_PADDING;

        let glossary = Glossary::new();
        let term_offset = 0.1;
        let definition_offset = 0.15;

        for (term, definition) in glossary.terms.iter().zip(glossary.definitions.iter()) {
            if self.need_new_page(
                y_fraction,
                term_line_height_fraction + definition_line_height_fraction,
            ) {
                self.new_page()?;
                y_fraction = 0.9;
            }

            self.add_text(term, self.bold_font, 12.0, term_offset, y_fraction, None)?;
            y_fraction -= term_line_height_fraction;

            // Set max width for glossary definitions as 70% of the page.
            let max_width = 0.9;
            let wrapped_lines =
                self.wrap_text(definition, definition_offset, max_width, self.font, 10.0);

            for line in wrapped_lines {
                if self.need_new_page(y_fraction, definition_line_height_fraction) {
                    self.new_page()?;
                    y_fraction = 0.9;
                }
                self.add_text(&line, self.font, 10.0, definition_offset, y_fraction, None)?;
                y_fraction -= definition_line_height_fraction;
            }

            y_fraction -= 0.5 * definition_line_height_fraction;
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

        path.line_to(
            PdfPoints::new(self.page_width * x2),
            PdfPoints::new(self.page_height * y2),
        )?;

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

    /// Adds the page numbers in the bottom right corner for each page.
    fn add_page_numbers(&mut self) -> Result<(), PdfError> {
        let total_pages = self.document.pages().len() as u32;
        let toc_pages = *self.section_page_map.get("Table of Contents").unwrap_or(&0) + 1;
        let mut current_page = 1;
        for page_index in toc_pages..total_pages {
            let text = format!("{}", current_page);

            let mut text_object =
                PdfPageTextObject::new(&self.document, &text, self.font, PdfPoints::new(12.0))?;
            text_object.set_fill_color(PdfColor::new(0, 0, 0, 255))?;
            text_object.translate(
                PdfPoints::new(self.page_width * 0.95),
                PdfPoints::new(self.page_height * 0.05),
            )?;

            let mut page = self.document.pages().get(page_index as u16).unwrap();
            page.objects_mut().add_text_object(text_object)?;

            current_page += 1;
        }

        Ok(())
    }

    /// Adds the dotted lines for the table of contents rows.
    fn add_dotted_line(&mut self, start_x: f32, end_x: f32, y: f32) -> Result<(), PdfError> {
        let total_width = end_x - start_x;
        let mut dotted_line = String::new();
        let mut current_width = 0.0;

        while current_width < total_width {
            dotted_line.push_str(".");
            current_width = self.get_text_width(&dotted_line, self.font, FONT_SIZE)?;
        }

        self.add_text(&dotted_line, self.font, FONT_SIZE, start_x, y, None)?;

        Ok(())
    }

    /// Calculates the width of the section heading for the table of contents. Used to calculate
    /// how long the dashed line should be. Width's are returned as a fraction of the page width.
    fn get_text_width(
        &self,
        text: &str,
        font: PdfFontToken,
        font_size: f32,
    ) -> Result<f32, PdfError> {
        let mut total_width = 0.0;
        let pdf_font = self.document.fonts().get(font).unwrap();

        let mut current_page = self.document.pages().get(self.current_page as u16).unwrap();

        let temp_object = current_page.objects_mut().create_text_object(
            PdfPoints::new(0.0),
            PdfPoints::new(0.0),
            text,
            pdf_font,
            PdfPoints::new(font_size),
        )?;

        if let Some(text_object) = temp_object.as_text_object() {
            let page_text = current_page.text()?;
            let chars = page_text.chars_for_object(text_object)?;

            for char in chars.iter() {
                if let Ok(bounds) = char.loose_bounds() {
                    total_width += bounds.width().value;
                }
            }
        }

        current_page.objects_mut().remove_object(temp_object)?;

        Ok(total_width / self.page_width)
    }

    /// Wrap text lines to prevent page overflows.
    fn wrap_text(
        &self,
        text: &str,
        offset: f32,
        max_width: f32,
        font: PdfFontToken,
        font_size: f32,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let words = text.split_whitespace();
        let available_width = max_width - offset;

        for word in words {
            let test_line = if current_line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current_line, word)
            };

            if self.get_text_width(&test_line, font, font_size).unwrap() <= available_width {
                current_line = test_line;
            } else {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                current_line = word.to_owned();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    // Helper function to add a filled rectangle
    fn add_rectangle(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: PdfColor,
    ) -> Result<(), PdfError> {
        let mut path = PdfPagePathObject::new(
            &self.document,
            PdfPoints::new(self.page_width * x1),
            PdfPoints::new(self.page_height * y1),
            Some(color),
            None,
            Some(color),
        )?;

        path.line_to(
            PdfPoints::new(self.page_width * x2),
            PdfPoints::new(self.page_height * y1),
        )?;
        path.line_to(
            PdfPoints::new(self.page_width * x2),
            PdfPoints::new(self.page_height * y2),
        )?;
        path.line_to(
            PdfPoints::new(self.page_width * x1),
            PdfPoints::new(self.page_height * y2),
        )?;
        path.close_path()?;

        let mut current_page = self.document.pages().get(self.current_page as u16).unwrap();
        current_page.objects_mut().add_path_object(path)?;
        Ok(())
    }
}

/// Converts a number to a roman numeral.
fn to_roman_numeral(num: u32) -> String {
    let symbols = [
        "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
    ];
    let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
    let mut result = String::new();
    let mut remaining = num;

    for (&symbol, &value) in symbols.iter().zip(values.iter()) {
        while remaining >= value {
            result.push_str(symbol);
            remaining -= value;
        }
    }

    result
}
