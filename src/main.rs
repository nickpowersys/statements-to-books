use crate::io_utils::glob_files_to_process;
use crate::parse_utils::{
    extract_card_purchase_captures_for_re, extract_deposit_captures_for_re,
    extract_online_payment_captures_for_re, extract_transfers_out_captures_for_re,
    parse_begin_or_end_bal_amt, parse_statement_begin_or_end_year, DebitCardPurchase, Deposit,
    OnlinePayment, TransferOut,
};
use crate::pyo3_pdf_service::{extract_text_from_page, get_page_count};
use chrono::Datelike;
use clap::Parser;
use fastnum::decimal::{Context, D256};
use regex::Regex;
use std::error::Error;
use std::ops::Range;
use std::path::PathBuf;

pub mod io_utils;
pub mod parse_utils;
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

    let statement_year_re = Regex::new(r"(?<begin_year>\d{4})\s+through\s").unwrap();
    let mut statement_year: i32 = 0;
    let mut one_indexed_page: usize;
    let begin_balance_re = Regex::new(r"(?m)^Beginning\sBalance.+[$](.+)$").unwrap();
    let mut begin_bal_usd: Option<fastnum::decimal::Decimal<4>> = None;
    let end_balance_re = Regex::new(r"(?m)^Ending\sBalance.+[$](.+)$").unwrap();
    let mut ending_bal_usd: Option<fastnum::decimal::Decimal<4>> = None;
    let mut page_deposits_vec: Vec<Deposit> = vec![];
    let mut deposits_vec: Vec<Deposit> = vec![];
    let mut transaction_month_str: &str;
    let mut transaction_month: u32;
    let mut transaction_day_str: &str;
    let mut transaction_day: u32;
    let mut transaction_year: i32;
    let mut transaction_amount: fastnum::decimal::Decimal<4>;
    let mut page_purchases_vec: Vec<DebitCardPurchase> = vec![];
    let mut card_purchases_vec: Vec<DebitCardPurchase> = vec![];
    let mut page_payments_vec: Vec<OnlinePayment> = vec![];
    let mut payments_vec: Vec<OnlinePayment> = vec![];
    let mut page_transfers_out_vec: Vec<TransferOut> = vec![];
    let mut transfers_out_vec: Vec<TransferOut> = vec![];

    let page_str_iter = pdf_page_strs.iter().enumerate();

    for (page_num, page_str) in page_str_iter {
        one_indexed_page = page_num + 1;
        println!("Parsing page {:#?}", one_indexed_page);
        if statement_year == 0 {
            if let Some(year_capture) = statement_year_re.captures_iter(page_str).next() {
                statement_year = parse_statement_begin_or_end_year(year_capture);
            };
        }
        if begin_bal_usd.is_none() {
            if let Some(bal_capture) = begin_balance_re.captures_iter(page_str).next() {
                begin_bal_usd = Some(parse_begin_or_end_bal_amt(bal_capture));
            }
        }
        if ending_bal_usd.is_none() {
            if let Some(bal_capture) = end_balance_re.captures_iter(page_str).next() {
                ending_bal_usd = Some(parse_begin_or_end_bal_amt(bal_capture));
            }
        }

        page_deposits_vec = extract_deposit_captures_for_re(page_str, statement_year);
        if !page_deposits_vec.is_empty() {
            for dep in &page_deposits_vec {
                transaction_month = dep.date.month();
                transaction_day = dep.date.day();
                transaction_year = dep.date.year();
                transaction_amount = dep.amount;
                println!(
                    "Page {one_indexed_page} Deposit {transaction_month}/{transaction_day}/{transaction_year} {transaction_amount:.2}",
                );
            }
            deposits_vec.extend(page_deposits_vec);
        }

        page_purchases_vec = extract_card_purchase_captures_for_re(page_str, statement_year);
        if !page_purchases_vec.is_empty() {
            for purch in &page_purchases_vec {
                transaction_month = purch.date.month();
                transaction_day = purch.date.day();
                transaction_year = purch.date.year();
                transaction_amount = purch.amount;
                println!(
                    "Page {one_indexed_page} Debit Card Purchase {transaction_month}/{transaction_day}/{transaction_year} {transaction_amount:.2}",
                );
            }
            card_purchases_vec.extend(page_purchases_vec);
        }

        page_payments_vec = extract_online_payment_captures_for_re(page_str, statement_year);
        if !page_payments_vec.is_empty() {
            for payment in &page_payments_vec {
                transaction_month = payment.date.month();
                transaction_day = payment.date.day();
                transaction_year = payment.date.year();
                transaction_amount = payment.amount;
                println!(
                    "Page {one_indexed_page} Online Payment {transaction_month}/{transaction_day}/{transaction_year} {transaction_amount:.2}",
                );
            }
            payments_vec.extend(page_payments_vec);
        }

        page_transfers_out_vec = extract_transfers_out_captures_for_re(page_str, statement_year);
        if !page_transfers_out_vec.is_empty() {
            for transfer in &page_transfers_out_vec {
                transaction_month = transfer.date.month();
                transaction_day = transfer.date.day();
                transaction_year = transfer.date.year();
                transaction_amount = transfer.amount;
                println!(
                    "Page {one_indexed_page} Transfer Out {transaction_month}/{transaction_day}/{transaction_year} {transaction_amount:.2}",
                );
            }
            transfers_out_vec.extend(page_transfers_out_vec);
        }

        // println!("Final deposits_vec {:#?}", deposits_vec);
        // println!("Final card_purchases_vec {:#?}", card_purchases_vec);
        // println!("Final payments_vec {:#?}", payments_vec);
    }
    if statement_year == 0 {
        panic!("Statement start year not parsed")
    }
    let revenue_usd: fastnum::decimal::Decimal<4> = if !deposits_vec.is_empty() {
        deposits_vec.iter().map(|invoice| invoice.amount).sum()
    } else {
        D256::from_str("0.00", Context::default()).unwrap()
    };

    let card_purchases_usd: fastnum::decimal::Decimal<4> = card_purchases_vec
        .iter()
        .map(|expense| expense.amount)
        .sum();
    let online_payments_usd: fastnum::decimal::Decimal<4> =
        payments_vec.iter().map(|expense| expense.amount).sum();
    let expenses_usd: fastnum::decimal::Decimal<4> = card_purchases_usd + online_payments_usd;
    let profit_usd = revenue_usd - expenses_usd;
    println!("Statement year {:>15}", format!("{:?}", statement_year));
    println!("Revenue {:>22}", format!("{:.2}", revenue_usd));
    println!("Expenses {:>21}", format!("{:.2}", expenses_usd));
    if revenue_usd > expenses_usd {
        println!("Profit {:>23}", format!("{:.2}", profit_usd));
    } else {
        println!("Loss {:>22}", format!("{:.2}", profit_usd));
    }
    let total_transfers_out: fastnum::decimal::Decimal<4> = if !transfers_out_vec.is_empty() {
        transfers_out_vec.iter().map(|invoice| invoice.amount).sum()
    } else {
        D256::from_str("0.00", Context::default()).unwrap()
    };
    if total_transfers_out > D256::from_str("0.00", Context::default()).unwrap() {
        println!(
            "Total Owner's Draws {:>10}",
            format!("{:.2}", total_transfers_out)
        );
    }

    let net_change_in_balance: fastnum::decimal::Decimal<4> =
        ending_bal_usd.unwrap() - begin_bal_usd.unwrap();
    let net_change_in_balance_based_on_transactions =
        revenue_usd - card_purchases_usd - online_payments_usd - total_transfers_out;

    if net_change_in_balance != net_change_in_balance_based_on_transactions {
        println!("Inflows and outflows and the profit/loss do not match up");
        println!("Net change in balance {:.2}", net_change_in_balance);
        println!(
            "Net profit/loss and transfers{:.2}",
            net_change_in_balance_based_on_transactions
        );
        println!(
            "Total mismatch {:.2}",
            net_change_in_balance - net_change_in_balance_based_on_transactions
        )
    }
}
