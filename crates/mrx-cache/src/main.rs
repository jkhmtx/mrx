use chrono as _;
use clap as _;
use mrx_cache::{
    Options,
    cache,
};
use mrx_utils as _;
use sqlx as _;
use thiserror as _;
use tokio as _;

#[tokio::main]
async fn main() {
    let (config, options) = Options::args().unwrap();

    cache(&config, &options).await.unwrap();
}
