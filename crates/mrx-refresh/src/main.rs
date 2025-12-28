use clap as _;
use mrx_utils as _;
use thiserror as _;

use mrx_refresh::{refresh, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    refresh(&config, &options);
}
