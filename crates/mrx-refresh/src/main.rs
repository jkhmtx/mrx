use clap as _;
use mrx_refresh::{
    Options,
    refresh,
};
use mrx_utils as _;
use thiserror as _;

fn main() {
    let (config, options) = Options::args().unwrap();
    refresh(&config, &options);
}
