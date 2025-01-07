use crate::io_utils::glob_files_to_process;
use crate::pyo3_pdf_service::{extract_text_from_page, get_page_count};
use clap::Parser;
use regex::{Match, Regex};
use std::error::Error;
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
    println!("Page count: {:?}", pg_count);
    let page_range: Range<u8> = 1..pg_count + 1;
    let mut pypdf_reader_page_index: u8;
    let mut raw_pg_text: Result<String, Box<dyn Error>>;
    let mut pdf_page_str: String;
    let mut pdf_page_strs: Vec<String> = Vec::new();

    for page_index in page_range {
        //println!("----Page {}----", page_index);
        pypdf_reader_page_index = page_index - 1;
        raw_pg_text = extract_text_from_page(&pdf_file_path, pypdf_reader_page_index);
        pdf_page_str = match raw_pg_text {
            Ok(pg_text_str) => pg_text_str,
            Err(e) => {
                println!("No text extracted from page {}. {:?}", page_index, e);
                continue;
            }
        };

        //println!("{}", pdf_page_str);
        pdf_page_strs.push(pdf_page_str);
    }
    let begin_balance_re = Regex::new(r"(?m)^Beginning\sBalance\s+[$]*(.+)$").unwrap();
    let mut begin_balance_raw: Option<Match> = None;
    let page_str_iter = pdf_page_strs.iter().enumerate();
    for page_str in page_str_iter {
        if begin_balance_raw.is_none() {
            println!("Parsing page {}", page_str.0 + 1);
            if let Some(line) = begin_balance_re.captures_iter(page_str.1).next() {
                begin_balance_raw = line.get(1);
            }
        } else {
            break;
        }
    }
    println!(
        "Beginning Balance raw: {:?}",
        begin_balance_raw.unwrap().as_str()
    );
}
