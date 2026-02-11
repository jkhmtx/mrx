use chrono as _;
use clap as _;
use mrx_cache::{
    Options,
    run,
};
use mrx_utils as _;
use sqlx as _;
use thiserror as _;
use tokio as _;

fn main() {
    let (config, options) = Options::args().unwrap();

    run(&config, &options);
}
