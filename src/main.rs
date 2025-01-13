use crate::io_utils::glob_files_to_process;
use crate::pyo3_pdf_service::{extract_text_from_page, get_page_count};
use clap::Parser;
use fastnum::{decimal::*, *};
use regex::{Captures, Match, Regex};
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
        println!("----Page {}----", page_index);
        pypdf_reader_page_index = page_index - 1;
        raw_pg_text = extract_text_from_page(&pdf_file_path, pypdf_reader_page_index);
        pdf_page_str = match raw_pg_text {
            Ok(pg_text_str) => pg_text_str,
            Err(e) => {
                println!("No text extracted from page {}. {:?}", page_index, e);
                continue;
            }
        };

        println!("{}", pdf_page_str);
        pdf_page_strs.push(pdf_page_str);
    }
    let begin_balance_re = Regex::new(r"(?m)^Beginning\sBalance.+[$](.+)$").unwrap();
    let end_balance_re = Regex::new(r"(?m)^Ending\sBalance.+[$](.+)$").unwrap();
    let wire_payment_re =
        Regex::new(r"(?ms)(?<date>\d{2}\/\d{2})\s(Orig\sCO\sName.+?)[$]?(?<amount_with_commas>[\d+[,]]*\d.\d\d)$")
            .unwrap();
    let card_purchase_re =
        Regex::new(r"(?ms)(?<date>\d{2}\/\d{2})\s(Recurring\sCard\sPurchase.+?)[$]?(?<amount_with_commas>[\d+[,]]*\d.\d\d)$")
            .unwrap();
    let mut begin_balance_line: Option<Match> = None;
    let mut end_balance_line: Option<Match> = None;
    let mut trans_byte_offset_opt: Option<usize>;
    //let mut deposit_trns: Match;
    let mut trans_captures: regex::Captures;
    let mut deposit_trans_captures_vec: Vec<regex::Captures> = vec![];
    let mut purchase_trans_captures_vec: Vec<regex::Captures> = vec![];
    //let mut deposit_trns_strs = Vec::<String>::new();
    let mut start_byte_offset: usize;
    let mut end_byte_offset: usize;
    let mut match_slice: &str;
    let page_str_iter = pdf_page_strs.iter().enumerate();
    fn extract_captures_for_re<'a>(
        re_expr: Regex,
        page_str: &'a str,
        captures_vec: &'a mut Vec<regex::Captures<'a>>,
    ) -> &'a Vec<regex::Captures<'a>> {
        let mut start_byte_offset: usize = 0;
        let mut trans_byte_offset_opt: Option<usize>;
        let mut end_byte_offset: usize;
        let mut match_slice: &str;
        let mut trans_captures: regex::Captures;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
        while trans_byte_offset_opt.is_some() {
            end_byte_offset = trans_byte_offset_opt.unwrap();
            match_slice = &page_str[start_byte_offset..end_byte_offset];
            trans_captures = re_expr
                .captures(match_slice)
                .expect(".is_some() must not be true");
            captures_vec.push(trans_captures);
            start_byte_offset = end_byte_offset + 1;
            trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
        }
        captures_vec
    };
    for (page_num, page_str) in page_str_iter {
        println!("Parsing page {}", page_num + 1);
        if begin_balance_line.is_none() {
            if let Some(line) = begin_balance_re.captures_iter(page_str).next() {
                begin_balance_line = line.get(1);
            }
        }
        if end_balance_line.is_none() {
            if let Some(line) = end_balance_re.captures_iter(page_str).next() {
                end_balance_line = line.get(1);
            }
        }

        start_byte_offset = 0;
        trans_byte_offset_opt = wire_payment_re.shortest_match_at(page_str, start_byte_offset);
        while trans_byte_offset_opt.is_some() {
            end_byte_offset = trans_byte_offset_opt.unwrap();
            match_slice = &page_str[start_byte_offset..end_byte_offset];
            trans_captures = wire_payment_re
                .captures(match_slice)
                .expect(".is_some() must not be true");
            deposit_trans_captures_vec.push(trans_captures);
            start_byte_offset = end_byte_offset + 1;
            trans_byte_offset_opt = wire_payment_re.shortest_match_at(page_str, start_byte_offset);
        }

        start_byte_offset = 0;
        trans_byte_offset_opt = card_purchase_re.shortest_match_at(page_str, start_byte_offset);
        while trans_byte_offset_opt.is_some() {
            end_byte_offset = trans_byte_offset_opt.unwrap();
            match_slice = &page_str[start_byte_offset..end_byte_offset];
            //deposit_trns = wire_payment_re
            //    .find(match_slice)
            //    .expect(".is_some() must not be true.");
            trans_captures = card_purchase_re
                .captures(match_slice)
                .expect(".is_some() must not be true");
            purchase_trans_captures_vec.push(trans_captures);
            //deposit_trns_strs.push(String::from(deposit_trns.as_str()));
            start_byte_offset = end_byte_offset + 1;
            trans_byte_offset_opt = wire_payment_re.shortest_match_at(page_str, start_byte_offset);
        }

        //if begin_balance_line.is_some() & end_balance_line.is_some() {
        //    break;
        //}
    }

    let mut beginning_balance_amt: String = String::from(begin_balance_line.unwrap().as_str());
    beginning_balance_amt.retain(|c| c != ',');
    let mut end_balance_amt: String = String::from(end_balance_line.unwrap().as_str());
    end_balance_amt.retain(|c| c != ',');

    #[derive(Debug)]
    enum TransactionKind {
        Debit,
        Credit,
    }

    #[derive(Clone)]
    struct Transaction<'a> {
        date: String,
        amount: fastnum::decimal::Decimal<4>,
        transaction_kind: &'a TransactionKind,
    }

    //let deposits: Vec<Transaction>;
    let begin_bal_usd: fastnum::decimal::Decimal<4> =
        D256::from_str(&beginning_balance_amt, Context::default()).unwrap();
    println!("begin_bal_usd {}", begin_bal_usd);
    let ending_bal_usd: fastnum::decimal::Decimal<4> =
        D256::from_str(&end_balance_amt, Context::default()).unwrap();
    println!("ending_bal_usd {}", ending_bal_usd);
    //println!("{:#?}", deposit_trans_captures_vec);
    //println!("{:#?}", purchase_trans_captures_vec);

    //let change_in_bal = end_bal_usd - begin_bal_usd;
    // let wire_deposit_amt_re = Regex::new(r"(?ms)^CO Entry\s+([$]*)(.+)$").unwrap();
    for deposit_captures in deposit_trans_captures_vec.iter() {
        println!("Deposit captures {:#?}", deposit_captures);
        // if let Some(cg) = end_balance_re.captures_iter(page_str.1).next() {
        //     line.get(1)
        // }
        // for (deposit_amt_raw) in wire_deposit_amt_re.find(&deposit_trns) {
        //     println!("Deposit amount {:?}", deposit_amt_raw);
        // }
    }
    for purchase_captures in purchase_trans_captures_vec.iter() {
        println!("Purchase captures: {:#?}", purchase_captures);
        // if let Some(cg) = end_balance_re.captures_iter(page_str.1).next() {
        //     line.get(1)
        // }
        // for (deposit_amt_raw) in wire_deposit_amt_re.find(&deposit_trns) {
        //     println!("Deposit amount {:?}", deposit_amt_raw);
        // }
    }

    let mut trans_date: String;
    let mut trans_amount_str: String;
    let mut trans_amount: fastnum::decimal::Decimal<4>;
    let mut deposits: Vec<Transaction> = vec![];
    let mut purchases: Vec<Transaction> = vec![];
    for deposit_captures in deposit_trans_captures_vec.iter() {
        trans_date = String::from(
            deposit_captures
                .name("date")
                .expect("already captured")
                .as_str(),
        );
        trans_amount_str = String::from(
            deposit_captures
                .name("amount_with_commas")
                .expect("already captured")
                .as_str(),
        );
        trans_amount_str.retain(|c| (c != ',') & (c != '$') & (c != ' '));
        println!(
            "cleaned deposit amount {:?} {:?}",
            trans_date, trans_amount_str
        );
        trans_amount = D256::from_str(&trans_amount_str, Context::default()).unwrap();

        deposits.push(Transaction {
            date: trans_date,
            amount: trans_amount,
            transaction_kind: &TransactionKind::Debit,
        })
    }

    fn statement_regex_captures_to_transactions<'a>(
        captures_vec: &Vec<Captures<'_>>,
        transaction_kind: &'a TransactionKind,
        transactions_vec: &'a mut Vec<Transaction<'a>>,
    ) -> &'a Vec<Transaction<'a>> {
        let mut trans_date: String;
        let mut trans_amount_str: String;
        let mut trans_amount: D256;
        for captures in captures_vec {
            trans_date = String::from(captures.name("date").expect("already captured").as_str());
            trans_amount_str = String::from(
                captures
                    .name("amount_with_commas")
                    .expect("already captured")
                    .as_str(),
            );
            trans_amount_str.retain(|c| (c != ',') & (c != '$') & (c != ' '));
            println!(
                "cleaned purchase amount {:?} {:?}",
                trans_date, trans_amount_str
            );
            trans_amount = D256::from_str(&trans_amount_str, Context::default()).unwrap();
            transactions_vec.push(Transaction {
                date: trans_date,
                amount: trans_amount,
                transaction_kind,
            });
        }
        transactions_vec
    }

    let mut purchases_populated: Vec<Transaction> = vec![];
    purchases_populated = statement_regex_captures_to_transactions(
        &purchase_trans_captures_vec,
        &TransactionKind::Credit,
        &mut purchases,
    )
    .to_vec();

    for purch in purchases_populated {
        println!(
            "{} {} {:#?}",
            purch.date, purch.amount, purch.transaction_kind
        );
    }

    // for purchase_captures in purchase_trans_captures_vec.iter() {
    //     trans_date = String::from(
    //         purchase_captures
    //             .name("date")
    //             .expect("already captured")
    //             .as_str(),
    //     );
    //     trans_amount_str = String::from(
    //         purchase_captures
    //             .name("amount_with_commas")
    //             .expect("already captured")
    //             .as_str(),
    //     );
    //     trans_amount_str.retain(|c| (c != ',') & (c != '$') & (c != ' '));
    //     println!(
    //         "cleaned purchase amount {:?} {:?}",
    //         trans_date, trans_amount_str
    //     );
    //     trans_amount = D256::from_str(&trans_amount_str, Context::default()).unwrap();
    //     purchase = Transaction {
    //         date: trans_date,
    //         amount: trans_amount,
    //         transaction_kind: &TransactionKind::Credit,
    //     };
    //     purchases.push(purchase);
    // }
    //println!("{:#?}", purchases_populated);
}
