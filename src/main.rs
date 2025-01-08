use crate::io_utils::glob_files_to_process;
use crate::pyo3_pdf_service::{extract_text_from_page, get_page_count};
use clap::Parser;
use regex::{Match, Regex};
use rusty_money::{iso, Money};
use std::error::Error;
use std::ops::Range;
use std::path::PathBuf;
use std::str;

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
    let begin_balance_re = Regex::new(r"(?m)^Beginning\sBalance.+[$](.+)$").unwrap();
    let end_balance_re = Regex::new(r"(?m)^Ending\sBalance.+[$](.+)$").unwrap();
    let wire_payment_re = Regex::new(r"(?ms)(\d{2}\/\d{2}.+?^CO Entry.+)$").unwrap();
    let mut begin_balance_line: Option<Match> = None;
    let mut end_balance_line: Option<Match> = None;
    let mut deposit_trns_byte_offset_opt: Option<usize>;
    let mut deposit_trns: Match;
    let mut deposit_trns_strs = Vec::<String>::new();
    let mut start_byte_offset: usize;
    let mut end_byte_offset: usize;
    let mut match_slice: &str;
    let page_str_iter = pdf_page_strs.iter().enumerate();
    for page_str in page_str_iter {
        println!("Parsing page {}", page_str.0 + 1);
        if begin_balance_line.is_none() {
            if let Some(line) = begin_balance_re.captures_iter(page_str.1).next() {
                begin_balance_line = line.get(1);
            }
        }
        if end_balance_line.is_none() {
            if let Some(line) = end_balance_re.captures_iter(page_str.1).next() {
                end_balance_line = line.get(1);
            }
        }

        start_byte_offset = 0;
        deposit_trns_byte_offset_opt =
            wire_payment_re.shortest_match_at(page_str.1, start_byte_offset);
        while deposit_trns_byte_offset_opt.is_some() {
            end_byte_offset = deposit_trns_byte_offset_opt.unwrap();

            match_slice = &page_str.1[start_byte_offset..end_byte_offset];
            deposit_trns = wire_payment_re
                .find(match_slice)
                .expect(".is_some() must not be true.");
            deposit_trns_strs.push(String::from(deposit_trns.as_str()));
            start_byte_offset = end_byte_offset + 1;
            deposit_trns_byte_offset_opt =
                wire_payment_re.shortest_match_at(page_str.1, start_byte_offset);
        }

        if begin_balance_line.is_some() & end_balance_line.is_some() {
            break;
        }
    }

    let mut beginning_balance_amt: String = String::from(begin_balance_line.unwrap().as_str());
    beginning_balance_amt.retain(|c| c != ',');
    let mut end_balance_amt: String = String::from(end_balance_line.unwrap().as_str());
    end_balance_amt.retain(|c| c != ',');
    let begin_bal_usd = Money::from_str(&beginning_balance_amt, iso::USD).unwrap();
    let end_bal_usd = Money::from_str(&end_balance_amt, iso::USD).unwrap();
    println!("Beginning Balance: {:?}", beginning_balance_amt);
    println!("Ending Balance: {:?}", end_balance_amt);
    println!("{:#?}", deposit_trns_strs);
    let change_in_bal = end_bal_usd - begin_bal_usd;
}
