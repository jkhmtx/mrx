use clap as _;
use mrx_show::{
    Options,
    show,
};
use mrx_utils as _;
use thiserror as _;

fn main() {
    let (config, options) = Options::args().unwrap();
    show(&config, &options);
}
