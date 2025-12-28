use clap as _;
use mrx_utils as _;
use thiserror as _;

use mrx_cache::{cache, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    cache(&config, &options).unwrap();
}
