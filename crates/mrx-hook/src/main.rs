use clap as _;
use mrx_hook::{
    Options,
    hook,
};
use mrx_utils as _;
use thiserror as _;

fn main() {
    let (config, options) = Options::args().unwrap();
    hook(&config, &options);
}
