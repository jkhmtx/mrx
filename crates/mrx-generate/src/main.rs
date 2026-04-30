use clap as _;
use exn as _;
use mrx_generate::{
    Options,
    run,
};
use mrx_utils as _;
use thiserror as _;

fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .compact()
        .with_writer(std::io::stderr)
        .init();

    let (config, options) = Options::args().unwrap();
    run(&config, &options);
}
