use mrx_utils::Config;

use crate::{
    Options,
    show::show,
};

pub fn run(config: &Config, options: &Options) {
    match show(config, options) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{e}");

            std::process::exit(1);
        }
    }
}
