use mrx_utils::Config;

use crate::{
    Options,
    hook::hook,
};

pub fn run(config: &Config, options: &Options) {
    match hook(config, options) {
        Ok(bins) => {
            eprintln!("The following commands are available in your shell:");
            for bin in bins {
                eprintln!("  - {bin}");
            }
        }
        Err(e) => {
            eprintln!("{e:?}");

            std::process::exit(1);
        }
    }
}
