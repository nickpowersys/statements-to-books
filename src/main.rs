use crate::io_utils::glob_files_to_process;
use clap::Parser;
use pyo3::prelude::*;
use std::path::PathBuf;

pub mod io_utils;

#[derive(Parser)]
struct Cli {
    pdf_dir: String,
    txt_dir: String,
}

fn main() {
    let args = Cli::parse();
    let input_file_ext = "pdf";
    let pdf_file_paths: Vec<PathBuf> = glob_files_to_process(&args.pdf_dir, input_file_ext)?;
    print!("{:?}", pdf_file_paths);

    fn get_page_count(pdf_file_paths: Vec<PathBuf>) -> Result<f64, Box<dyn std::error::Error>> {
        Python::with_gil(|py| {
            let pdf_parser = PyModule::import(py, "statements_to_books.pdf_parser")
                .expect("unable to import 'pdf_parser'")
                .getattr("loads")
                .unwrap();
            let result: f64 = pdf_parser
                .getattr("page_count_of_pdf")?
                .call1((&pdf_file_paths[0],))?
                .extract()?;
            Ok(result)
        })
    }

    let pg_count = get_page_count(pdf_file_paths);
    println!("{:?}", pg_count);
}
