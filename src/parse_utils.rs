use chrono::{Datelike, NaiveDate};
use fastnum::{decimal::*, D256};
use regex::{Captures, Regex};

#[derive(Debug)]
pub enum TransactionKind {
    Debit,
    Credit,
}

trait TransactionKindConst {
    const DEBIT_OR_CREDIT: TransactionKind;
}

struct TransactionProps {
    pub date: String,
    pub amount: fastnum::decimal::Decimal<4>,
}

#[derive(Debug)]
pub(crate) struct Deposit {
    pub date: NaiveDate,
    pub amount: fastnum::decimal::Decimal<4>,
}

impl TransactionKindConst for Deposit {
    const DEBIT_OR_CREDIT: TransactionKind = TransactionKind::Credit;
}

impl Deposit {
    pub fn new(date: NaiveDate, raw_amount: &mut String) -> Self {
        raw_amount.retain(|c| c != ',');
        Self {
            date,
            amount: D256::from_str(raw_amount, Context::default()).unwrap(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct DebitCardPurchase {
    pub date: NaiveDate,
    pub amount: fastnum::decimal::Decimal<4>,
}

impl TransactionKindConst for DebitCardPurchase {
    const DEBIT_OR_CREDIT: TransactionKind = TransactionKind::Debit;
}

impl DebitCardPurchase {
    pub fn new(date: NaiveDate, raw_amount: &mut String) -> Self {
        raw_amount.retain(|c| c != ',');
        Self {
            date,
            amount: D256::from_str(raw_amount, Context::default()).unwrap(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct OnlinePayment {
    pub date: NaiveDate,
    pub amount: fastnum::decimal::Decimal<4>,
}

impl TransactionKindConst for OnlinePayment {
    const DEBIT_OR_CREDIT: TransactionKind = TransactionKind::Debit;
}

impl OnlinePayment {
    pub fn new(date: NaiveDate, raw_amount: &mut String) -> Self {
        raw_amount.retain(|c| c != ',');
        Self {
            date,
            amount: D256::from_str(raw_amount, Context::default()).unwrap(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TransferOut {
    pub date: NaiveDate,
    pub amount: fastnum::decimal::Decimal<4>,
}

impl TransactionKindConst for TransferOut {
    const DEBIT_OR_CREDIT: TransactionKind = TransactionKind::Debit;
}

impl TransferOut {
    pub fn new(date: NaiveDate, raw_amount: &mut String) -> Self {
        raw_amount.retain(|c| c != ',');
        Self {
            date,
            amount: D256::from_str(raw_amount, Context::default()).unwrap(),
        }
    }
}

pub(crate) fn parse_statement_begin_or_end_year(year_capture: Captures) -> i32 {
    let year_capture_match = year_capture.get(1);
    let mut year_str = String::from(year_capture_match.unwrap().as_str());
    year_str.parse::<i32>().expect("Not parsed")
}

pub(crate) fn parse_begin_or_end_bal_amt(bal_captures: Captures) -> Decimal<4> {
    let bal_capture_match = bal_captures.get(1);
    let mut bal_str = String::from(bal_capture_match.unwrap().as_str());
    bal_str.retain(|c| c != ',');
    D256::from_str(&bal_str, Context::default()).unwrap()
}

pub(crate) fn extract_deposit_captures_for_re(page_str: &str, statement_year: i32) -> Vec<Deposit> {
    let wire_payment_re =
    Regex::new(r"(?ms)(?<date>\d{2}\/\d{2})\s(Orig\sCO\sName.+?)(Descr:Payments)(.+?)[$]?(?<amount_with_commas>[\d+[,]]*\d.\d\d)$")
        .unwrap();
    let re_expr: Regex = wire_payment_re;
    let mut start_byte_offset: usize = 0;
    let mut trans_byte_offset_opt: Option<usize>;
    let mut end_byte_offset: usize;
    let mut match_slice: &str;
    let mut captures: Captures;
    let mut raw_amount: String;
    let mut transaction_month_day_str: String;
    let mut transaction_month_str: &str;
    let mut transaction_month: u32;
    let mut transaction_day_str: &str;
    let mut transaction_day: u32;
    let current_date = chrono::Utc::now();
    let current_year = current_date.year();
    let mut transaction_year: i32;
    let mut transaction_date: NaiveDate;
    let mut deposit: Deposit;
    let mut deposits: Vec<Deposit> = vec![];

    trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    while trans_byte_offset_opt.is_some() {
        end_byte_offset = trans_byte_offset_opt.unwrap();
        match_slice = &page_str[start_byte_offset..end_byte_offset];
        captures = re_expr
            .captures(match_slice)
            .expect(".is_some() must not be true");
        raw_amount = String::from(
            captures
                .name("amount_with_commas")
                .expect("already captured")
                .as_str(),
        );
        transaction_month_day_str =
            String::from(captures.name("date").expect("already_captured").as_str());
        let Some((transaction_month_str, transaction_day_str)) =
            transaction_month_day_str.split_once("/")
        else {
            panic!(
                "transaction_month_day_str {} not parsed.",
                transaction_month_day_str
            );
        };
        transaction_month = transaction_month_str
            .parse::<u32>()
            .expect("Not valid month string");
        transaction_day = transaction_day_str
            .parse::<u32>()
            .expect("Not valid day string");
        if statement_year != 0 {
            transaction_year = statement_year;
        } else {
            transaction_year = current_year;
        }
        transaction_date =
            NaiveDate::from_ymd_opt(transaction_year, transaction_month, transaction_day)
                .expect("from_ymd_opt() failed.");
        deposit = Deposit::new(transaction_date, &mut raw_amount);
        deposits.push(deposit);
        start_byte_offset = end_byte_offset + 1;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    }
    deposits
}

pub(crate) fn extract_card_purchase_captures_for_re(
    page_str: &str,
    statement_year: i32,
) -> Vec<DebitCardPurchase> {
    let card_purchase_re =
    Regex::new(r"(?ms)(?<date>\d{2}\/\d{2})\s(Recurring\sCard\sPurchase.+?)[$]?(?<amount_with_commas>[\d+[,]]*\d.\d\d)$")
        .unwrap();
    let re_expr: Regex = card_purchase_re;
    let mut start_byte_offset: usize = 0;
    let mut trans_byte_offset_opt: Option<usize>;
    let mut end_byte_offset: usize;
    let mut match_slice: &str;
    let mut captures: Captures;
    let mut raw_amount: String;
    let mut transaction_month_day_str: String;
    let mut transaction_month_str: &str;
    let mut transaction_month: u32;
    let mut transaction_day_str: &str;
    let mut transaction_day: u32;
    let current_date = chrono::Utc::now();
    let current_year = current_date.year();
    let mut transaction_year: i32;
    let mut transaction_date: NaiveDate;
    let mut purchase: DebitCardPurchase;
    let mut purchases: Vec<DebitCardPurchase> = vec![];
    trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    while trans_byte_offset_opt.is_some() {
        end_byte_offset = trans_byte_offset_opt.unwrap();
        match_slice = &page_str[start_byte_offset..end_byte_offset];
        captures = re_expr
            .captures(match_slice)
            .expect(".is_some() must not be true");
        raw_amount = String::from(
            captures
                .name("amount_with_commas")
                .expect("already captured")
                .as_str(),
        );
        transaction_month_day_str =
            String::from(captures.name("date").expect("already_captured").as_str());
        let Some((transaction_month_str, transaction_day_str)) =
            transaction_month_day_str.split_once("/")
        else {
            panic!(
                "transaction_month_day_str {} not parsed.",
                transaction_month_day_str
            );
        };
        transaction_month = transaction_month_str
            .parse::<u32>()
            .expect("Not valid month string");
        transaction_day = transaction_day_str
            .parse::<u32>()
            .expect("Not valid day string");
        if statement_year != 0 {
            transaction_year = statement_year;
        } else {
            transaction_year = current_year;
        }
        transaction_date =
            NaiveDate::from_ymd_opt(transaction_year, transaction_month, transaction_day)
                .expect("from_ymd_opt() failed.");
        purchase = DebitCardPurchase::new(transaction_date, &mut raw_amount);
        purchases.push(purchase);
        start_byte_offset = end_byte_offset + 1;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    }
    purchases
}

pub(crate) fn extract_online_payment_captures_for_re(
    page_str: &str,
    statement_year: i32,
) -> Vec<OnlinePayment> {
    let online_payment_re =
    Regex::new(r"(?ms)(?<date>\d{2}\/\d{2})\s(.+?)(Xfer)(.+?)[$]?(?<amount_with_commas>[\d+[,]]*\d.\d\d).?$")
        .unwrap();
    let re_expr: Regex = online_payment_re;
    let mut start_byte_offset: usize = 0;
    let mut trans_byte_offset_opt: Option<usize>;
    let mut end_byte_offset: usize;
    let mut match_slice: &str;
    let mut captures: Captures;
    let mut raw_amount: String;
    let mut transaction_month_day_str: String;
    let mut transaction_month_str: &str;
    let mut transaction_month: u32;
    let mut transaction_day_str: &str;
    let mut transaction_day: u32;
    let current_date = chrono::Utc::now();
    let current_year = current_date.year();
    let mut transaction_year: i32;
    let mut transaction_date: NaiveDate;
    let mut payment: OnlinePayment;
    let mut payments: Vec<OnlinePayment> = vec![];
    trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    while trans_byte_offset_opt.is_some() {
        end_byte_offset = trans_byte_offset_opt.unwrap();
        match_slice = &page_str[start_byte_offset..end_byte_offset];
        captures = re_expr
            .captures(match_slice)
            .expect(".is_some() must not be true");
        raw_amount = String::from(
            captures
                .name("amount_with_commas")
                .expect("already captured")
                .as_str(),
        );
        transaction_month_day_str =
            String::from(captures.name("date").expect("already_captured").as_str());
        let Some((transaction_month_str, transaction_day_str)) =
            transaction_month_day_str.split_once("/")
        else {
            panic!(
                "transaction_month_day_str {} not parsed.",
                transaction_month_day_str
            );
        };
        transaction_month = transaction_month_str
            .parse::<u32>()
            .expect("Not valid month string");
        transaction_day = transaction_day_str
            .parse::<u32>()
            .expect("Not valid day string");
        if statement_year != 0 {
            transaction_year = statement_year;
        } else {
            transaction_year = current_year;
        }
        transaction_date =
            NaiveDate::from_ymd_opt(transaction_year, transaction_month, transaction_day)
                .expect("from_ymd_opt() failed.");
        payment = OnlinePayment::new(transaction_date, &mut raw_amount);
        payments.push(payment);
        start_byte_offset = end_byte_offset + 1;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    }
    payments
}

pub(crate) fn extract_transfers_out_captures_for_re(
    page_str: &str,
    statement_year: i32,
) -> Vec<TransferOut> {
    let transfer_out_re =
    Regex::new(r"(?ms)(?<date>\d{2}\/\d{2})\s(.+?)(Online\sTransfer\sTo)(.+?)[$]?(?<amount_with_commas>[\d+[,]]*\d.\d\d).?$")
        .unwrap();
    let re_expr: Regex = transfer_out_re;
    let mut start_byte_offset: usize = 0;
    let mut trans_byte_offset_opt: Option<usize>;
    let mut end_byte_offset: usize;
    let mut match_slice: &str;
    let mut captures: Captures;
    let mut raw_amount: String;
    let mut transaction_month_day_str: String;
    let mut transaction_month_str: &str;
    let mut transaction_month: u32;
    let mut transaction_day_str: &str;
    let mut transaction_day: u32;
    let current_date = chrono::Utc::now();
    let current_year = current_date.year();
    let mut transaction_year: i32;
    let mut transaction_date: NaiveDate;
    let mut transfer: TransferOut;
    let mut transfers: Vec<TransferOut> = vec![];
    trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    while trans_byte_offset_opt.is_some() {
        end_byte_offset = trans_byte_offset_opt.unwrap();
        match_slice = &page_str[start_byte_offset..end_byte_offset];
        captures = re_expr
            .captures(match_slice)
            .expect(".is_some() must not be true");
        raw_amount = String::from(
            captures
                .name("amount_with_commas")
                .expect("already captured")
                .as_str(),
        );
        transaction_month_day_str =
            String::from(captures.name("date").expect("already_captured").as_str());
        let Some((transaction_month_str, transaction_day_str)) =
            transaction_month_day_str.split_once("/")
        else {
            panic!(
                "transaction_month_day_str {} not parsed.",
                transaction_month_day_str
            );
        };
        transaction_month = transaction_month_str
            .parse::<u32>()
            .expect("Not valid month string");
        transaction_day = transaction_day_str
            .parse::<u32>()
            .expect("Not valid day string");
        if statement_year != 0 {
            transaction_year = statement_year;
        } else {
            transaction_year = current_year;
        }
        transaction_date =
            NaiveDate::from_ymd_opt(transaction_year, transaction_month, transaction_day)
                .expect("from_ymd_opt() failed.");
        transfer = TransferOut::new(transaction_date, &mut raw_amount);
        transfers.push(transfer);
        start_byte_offset = end_byte_offset + 1;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    }
    transfers
}
