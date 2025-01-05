use crate::io_utils::glob_files_to_process;
use crate::pyo3_pdf_service::get_page_count;
use clap::Parser;
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
    let pg_count = get_page_count(pdf_file_path);
    println!("{:?}", pg_count);
}
