use pyo3::prelude::*;
use std::path::PathBuf;

pub(crate) fn get_page_count(pdf_file_path: &PathBuf) -> Result<u8, Box<dyn std::error::Error>> {
    Python::with_gil(|py| {
        let pdf_parser = PyModule::import(py, "statements_to_books.pdf_parser")
            .expect("unable to import 'pdf_parser'");
        let result: u8 = pdf_parser
            .getattr("page_count_of_pdf")?
            .call1((pdf_file_path,))?
            .extract()?;
        Ok(result)
    })
}

pub(crate) fn extract_text_from_page(
    pdf_file_path: &PathBuf,
    page: u8,
) -> Result<String, Box<dyn std::error::Error>> {
    Python::with_gil(|py| {
        let pdf_parser = PyModule::import(py, "statements_to_books.pdf_parser")
            .expect("unable to import 'pdf_parser'");
        let result: String = pdf_parser
            .getattr("extract_text_from_pdf")?
            .call1((&pdf_file_path, page))?
            .extract()?;
        Ok(result)
    })
}
