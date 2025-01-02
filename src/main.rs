use crate::io_utils::glob_files_to_process;
use clap::Parser;
use glob::PatternError;
use std::path::PathBuf;

pub mod io_utils;

#[derive(Parser)]
struct Cli {
    pdf_dir: String,
    txt_dir: String,
}

fn main() {
    let args = Cli::parse();
    println!("The pdf_dir is {}!", &args.pdf_dir);
    println!("The txt_dir is {}!", &args.txt_dir);
    let input_file_ext = "txt";
    let pdf_file_paths: Result<Vec<PathBuf>, PatternError> =
        Ok(glob_files_to_process(&args.pdf_dir, input_file_ext).unwrap());
}
