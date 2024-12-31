use clap::Parser;

#[derive(Parser)]
struct Cli {
    pdf_dir: String,
    txt_dir: String,
}

fn main() {
    let args = Cli::parse();
    println!("The pdf_dir is {}!", &args.pdf_dir);
    println!("The txt_dir is {}!", &args.txt_dir);
}
