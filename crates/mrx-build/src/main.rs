use clap as _;
use mrx_build::{
    Options,
    build,
};
use mrx_utils as _;
use thiserror as _;

fn main() {
    let (config, options) = Options::args().unwrap();
    build(&config, &options).unwrap();
}
