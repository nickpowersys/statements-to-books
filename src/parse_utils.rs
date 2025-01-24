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
    pub date: String,
    pub amount: fastnum::decimal::Decimal<4>,
}

impl TransactionKindConst for Deposit {
    const DEBIT_OR_CREDIT: TransactionKind = TransactionKind::Credit;
}

impl Deposit {
    pub fn new(date: String, raw_amount: &mut String) -> Self {
        raw_amount.retain(|c| c != ',');
        Self {
            date,
            amount: D256::from_str(raw_amount, Context::default()).unwrap(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct DebitCardPurchase {
    pub date: String,
    pub amount: fastnum::decimal::Decimal<4>,
}

impl TransactionKindConst for DebitCardPurchase {
    const DEBIT_OR_CREDIT: TransactionKind = TransactionKind::Debit;
}

impl DebitCardPurchase {
    pub fn new(date: String, raw_amount: &mut String) -> Self {
        raw_amount.retain(|c| c != ',');
        Self {
            date,
            amount: D256::from_str(raw_amount, Context::default()).unwrap(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct OnlinePayment {
    pub date: String,
    pub amount: fastnum::decimal::Decimal<4>,
}

impl TransactionKindConst for OnlinePayment {
    const DEBIT_OR_CREDIT: TransactionKind = TransactionKind::Debit;
}

impl OnlinePayment {
    pub fn new(date: String, raw_amount: &mut String) -> Self {
        raw_amount.retain(|c| c != ',');
        Self {
            date,
            amount: D256::from_str(raw_amount, Context::default()).unwrap(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TransferOut {
    pub date: String,
    pub amount: fastnum::decimal::Decimal<4>,
}

impl TransactionKindConst for TransferOut {
    const DEBIT_OR_CREDIT: TransactionKind = TransactionKind::Debit;
}

impl TransferOut {
    pub fn new(date: String, raw_amount: &mut String) -> Self {
        raw_amount.retain(|c| c != ',');
        Self {
            date,
            amount: D256::from_str(raw_amount, Context::default()).unwrap(),
        }
    }
}

pub(crate) fn parse_begin_or_end_bal_amt(bal_captures: Captures) -> Decimal<4> {
    let bal_capture_match = bal_captures.get(1);
    let mut bal_str = String::from(bal_capture_match.unwrap().as_str());
    bal_str.retain(|c| c != ',');
    D256::from_str(&bal_str, Context::default()).unwrap()
}

pub(crate) fn extract_deposit_captures_for_re(page_str: &str) -> Vec<Deposit> {
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
        deposit = Deposit::new(
            String::from(captures.name("date").expect("already_captured").as_str()),
            &mut raw_amount,
        );
        deposits.push(deposit);
        start_byte_offset = end_byte_offset + 1;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    }
    deposits
}

pub(crate) fn extract_card_purchase_captures_for_re(page_str: &str) -> Vec<DebitCardPurchase> {
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
        purchase = DebitCardPurchase::new(
            String::from(captures.name("date").expect("already_captured").as_str()),
            &mut raw_amount,
        );
        purchases.push(purchase);
        start_byte_offset = end_byte_offset + 1;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    }
    purchases
}

pub(crate) fn extract_online_payment_captures_for_re(page_str: &str) -> Vec<OnlinePayment> {
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
        payment = OnlinePayment::new(
            String::from(captures.name("date").expect("already_captured").as_str()),
            &mut raw_amount,
        );
        payments.push(payment);
        start_byte_offset = end_byte_offset + 1;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    }
    payments
}

pub(crate) fn extract_transfers_out_captures_for_re(page_str: &str) -> Vec<TransferOut> {
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
        transfer = TransferOut::new(
            String::from(captures.name("date").expect("already_captured").as_str()),
            &mut raw_amount,
        );
        transfers.push(transfer);
        start_byte_offset = end_byte_offset + 1;
        trans_byte_offset_opt = re_expr.shortest_match_at(page_str, start_byte_offset);
    }
    transfers
}
