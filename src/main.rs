use clap::Parser;
use std::result::Result;
use std::error::Error;

use glimpse::processing::args::Args;
use glimpse::processing::runner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    return runner::run(&args).await;
}