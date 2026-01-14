use clap as _;
use mrx_build::{
    Options,
    run,
};
use mrx_generate as _;
use mrx_hook as _;
use mrx_utils as _;
use thiserror as _;

fn main() {
    let (config, options) = Options::args().unwrap();
    run(&config, &options);
}
