//! # PDF Base Module
//!

use pdfium_render::prelude::*;
use thiserror::Error;

/// The error types for the pdf modules.
#[derive(Error, Debug)]
pub enum PdfError {
    /// Occurs on a Pdfium library error.
    #[error("Pdf error: {0}")]
    Pdfium(#[from] pdfium_render::error::PdfiumError),
}

/// 
pub fn new_pdf(pdfium: &Pdfium) -> Result<PdfDocument, PdfError> {
    let document = pdfium.create_new_pdf()?;
    Ok(document)
}

/// Creates
pub fn create_title_page(document: &mut PdfDocument) -> Result<u8, PdfError> {
    let mut page = document.pages_mut().create_page_at_start(PdfPagePaperSize::a4())?;

    Ok(0)
}
