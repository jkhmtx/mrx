use clap as _;
use mrx_cache::{
    Options,
    cache,
};
use mrx_utils as _;
use thiserror as _;

fn main() {
    let (config, options) = Options::args().unwrap();
    cache(&config, &options).unwrap();
}
