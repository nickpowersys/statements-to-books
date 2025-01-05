use pyo3::prelude::*;
use std::path::PathBuf;

pub(crate) fn get_page_count(pdf_file_path: PathBuf) -> Result<f64, Box<dyn std::error::Error>> {
    Python::with_gil(|py| {
        let pdf_parser = PyModule::import(py, "statements_to_books.pdf_parser")
            .expect("unable to import 'pdf_parser'");
        let result: f64 = pdf_parser
            .getattr("page_count_of_pdf")?
            .call1((&pdf_file_path,))?
            .extract()?;
        Ok(result)
    })
}
