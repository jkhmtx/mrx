use clap as _;
use mrx_utils as _;
use thiserror as _;

use mrx_hook::{hook, Options};

fn main() {
    let (config, options) = Options::args().unwrap();
    hook(&config, &options);
}
