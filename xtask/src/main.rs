use std::process;

use clap::Parser;
use console::style;
use sqlx_cli::Opt;

#[derive(Parser, Debug)]
#[clap(bin_name = "cargo xtask")]
enum Cli {
    Sqlx(Opt),
}

#[tokio::main]
async fn main() {
    let Cli::Sqlx(opt) = Cli::parse();

    if let Err(error) = sqlx_cli::run(opt).await {
        eprintln!("{} {}", style("error:").bold().red(), error);
        process::exit(1);
    }
}
