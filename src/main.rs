use crate::io_utils::glob_files_to_process;
use crate::pyo3_pdf_service::{extract_text_from_page, get_page_count};
use clap::Parser;
use std::ops::Range;
use std::path::PathBuf;

pub mod io_utils;
pub mod pyo3_pdf_service;

#[derive(Parser)]
struct Cli {
    pdf_dir: String,
    txt_dir: String,
}

fn main() {
    let args = Cli::parse();
    let input_file_ext = "pdf";
    let pdf_file_paths: Vec<PathBuf> =
        glob_files_to_process(&args.pdf_dir, input_file_ext).unwrap();
    println!("{:?}", pdf_file_paths);

    let pdf_file_path: PathBuf = pdf_file_paths[0].clone();
    pyo3::prepare_freethreaded_python();
    let pg_count: u8 = get_page_count(&pdf_file_path).unwrap();
    println!("{:?}", pg_count);
    let page_range: Range<u8> = 1..pg_count;
    for page_index in page_range {
        let pypdf_reader_page_index: u8 = page_index - 1;
        let pg_text = extract_text_from_page(&pdf_file_path, pypdf_reader_page_index);
        println!("{}", page_index);
        println!("{:?}", pg_text);
    }
}
