##  Motivation and Description

I wanted to extract the transactions from my business checking account and determine the profit based on revenues and expenses.

I found that my bank does not provide statements in a delimited text format that can be imported easily.

The statements-to-books application is a CLI built with PyO3/maturin.

The source is written primarily in Rust.

## How Account Statements are Processed

Rust calls Python to extract raw text from each pdf page.

The starting and ending balance, as well as all transaction types found on a sample statement, are extracted from the raw text with Rust regex captures.

Each transaction is converted to a struct that contains the date and amount of the transaction. The structs are appended to vectors.

Each transaction type represents either a debit or credit. Based on the transaction type amounts and the combined transactions for each type, the net change in balance is calculated.

Finally, the calculated net change in balance is compared with the net change indicated by the starting and ending balance from the statement.

While the current version of the CLI displays transactions and calculated amounts, the functionality can be extended next by making the transactions and the calculated amounts persisent, and creating simple accounting statements after the extraction and validation.
