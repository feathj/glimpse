use clap::Parser;
use std::result::Result;
use std::error::Error;

mod processor;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    action: String,
    #[clap(short, long)]
    files: String,
    #[clap(short, long)]
    person_name: String,
    #[clap(short, long)]
    reference_file: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    return processor::run(&args).await;
}