use crate::io_utils::glob_files_to_process;
use crate::parse_utils::{
    extract_card_purchase_captures_for_re, extract_deposit_captures_for_re,
    parse_begin_or_end_bal_amt, DebitCardPurchase, Deposit, OnlinePayment,
};
use crate::pyo3_pdf_service::{extract_text_from_page, get_page_count};
use clap::Parser;
use fastnum::decimal::{Context, Decimal, D256};
use parse_utils::{
    extract_online_payment_captures_for_re, extract_transfers_out_captures_for_re, TransferOut,
};
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
    let begin_balance_re = Regex::new(r"(?m)^Beginning\sBalance.+[$](.+)$").unwrap();
    let mut begin_bal_usd: Option<fastnum::decimal::Decimal<4>> = None;
    let end_balance_re = Regex::new(r"(?m)^Ending\sBalance.+[$](.+)$").unwrap();
    let mut ending_bal_usd: Option<fastnum::decimal::Decimal<4>> = None;
    let mut page_deposits_vec: Vec<Deposit> = vec![];
    let mut deposits_vec: Vec<Deposit> = vec![];
    let mut page_purchases_vec: Vec<DebitCardPurchase> = vec![];
    let mut card_purchases_vec: Vec<DebitCardPurchase> = vec![];
    let mut page_payments_vec: Vec<OnlinePayment> = vec![];
    let mut payments_vec: Vec<OnlinePayment> = vec![];
    let mut page_transfers_out_vec: Vec<TransferOut> = vec![];
    let mut transfers_out_vec: Vec<TransferOut> = vec![];

    let page_str_iter = pdf_page_strs.iter().enumerate();

    for (page_num, page_str) in page_str_iter {
        println!("Parsing page {}", page_num + 1);
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

        page_deposits_vec = extract_deposit_captures_for_re(page_str);
        if !page_deposits_vec.is_empty() {
            for dep in &page_deposits_vec {
                println!(
                    "Page {} Deposit {}: {:.2}",
                    page_num + 1,
                    dep.date,
                    dep.amount
                );
            }
            deposits_vec.extend(page_deposits_vec);
        }

        page_purchases_vec = extract_card_purchase_captures_for_re(page_str);
        if !page_purchases_vec.is_empty() {
            for purch in &page_purchases_vec {
                println!(
                    "Page {} Purchase {}: {:.2}",
                    page_num + 1,
                    purch.date,
                    purch.amount
                );
            }
            card_purchases_vec.extend(page_purchases_vec);
        }

        page_payments_vec = extract_online_payment_captures_for_re(page_str);
        if !page_payments_vec.is_empty() {
            for payment in &page_payments_vec {
                println!(
                    "Page {} Payment {}: {:.2}",
                    page_num + 1,
                    payment.date,
                    payment.amount
                );
            }
            payments_vec.extend(page_payments_vec);
        }

        page_transfers_out_vec = extract_transfers_out_captures_for_re(page_str);
        if !page_transfers_out_vec.is_empty() {
            for transfer in &page_transfers_out_vec {
                println!(
                    "Page {} Transfer out {}: {:.2}",
                    page_num + 1,
                    transfer.date,
                    transfer.amount
                );
            }
            transfers_out_vec.extend(page_transfers_out_vec);
        }

        // println!("Final deposits_vec {:#?}", deposits_vec);
        // println!("Final card_purchases_vec {:#?}", card_purchases_vec);
        // println!("Final payments_vec {:#?}", payments_vec);
    }
    let revenue_usd: fastnum::decimal::Decimal<4> = if !deposits_vec.is_empty() {
        deposits_vec.iter().map(|invoice| invoice.amount).sum()
    } else {
        D256::from_str("0.00", Context::default()).unwrap()
    };

    // deposits_vec.iter().map(|invoice| invoice.amount).sum();
    let card_purchases_usd: fastnum::decimal::Decimal<4> = card_purchases_vec
        .iter()
        .map(|expense| expense.amount)
        .sum();
    let online_payments_usd: fastnum::decimal::Decimal<4> =
        payments_vec.iter().map(|expense| expense.amount).sum();
    let expenses_usd: fastnum::decimal::Decimal<4> = card_purchases_usd + online_payments_usd;
    let profit_usd = revenue_usd - expenses_usd;
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

    // #[derive(Debug)]
    // enum TransactionKind {
    //     Debit,
    //     Credit,
    // }

    // #[derive(Clone)]
    // struct Transaction<'a> {
    //     date: String,
    //     amount: fastnum::decimal::Decimal<4>,
    //     transaction_kind: &'a TransactionKind,
    // }

    //let change_in_bal = end_bal_usd - begin_bal_usd;

    // fn statement_regex_captures_to_transactions<'a>(
    //     captures_vec: &Vec<Captures<'_>>,
    //     transaction_kind: &'a TransactionKind,
    //     transactions_vec: &'a mut Vec<Transaction<'a>>,
    // ) -> &'a Vec<Transaction<'a>> {
    //     let mut trans_date: String;
    //     let mut trans_amount_str: String;
    //     let mut trans_amount: D256;
    //     for captures in captures_vec {
    //         trans_date = String::from(captures.name("date").expect("already captured").as_str());
    //         trans_amount_str = String::from(
    //             captures
    //                 .name("amount_with_commas")
    //                 .expect("already captured")
    //                 .as_str(),
    //         );
    //         trans_amount_str.retain(|c| (c != ',') & (c != '$') & (c != ' '));
    //         println!(
    //             "cleaned purchase amount {:?} {:?}",
    //             trans_date, trans_amount_str
    //         );
    //         trans_amount = D256::from_str(&trans_amount_str, Context::default()).unwrap();
    //         transactions_vec.push(Transaction {
    //             date: trans_date,
    //             amount: trans_amount,
    //             transaction_kind,
    //         });
    //     }
    //     transactions_vec
    // }

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
