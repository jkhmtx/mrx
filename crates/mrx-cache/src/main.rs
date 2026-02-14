use clap as _;
use mrx_cache::{
    Options,
    run,
};
use mrx_utils as _;
use rusqlite as _;
use thiserror as _;

fn main() {
    let (config, options) = Options::args().unwrap();

    run(&config, &options);
}
