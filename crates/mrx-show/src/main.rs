use clap as _;
use mrx_utils as _;
use thiserror as _;

use mrx_show::{show, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    show(&config, &options);
}
