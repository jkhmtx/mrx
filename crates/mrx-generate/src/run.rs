use mrx_utils::Config;

use crate::{
    Options,
    generate::generate,
};

pub fn run(config: &Config, options: &Options) {
    if let Err(e) = generate(config, options) {
        eprintln!("{e}");

        std::process::exit(1);
    }
}
