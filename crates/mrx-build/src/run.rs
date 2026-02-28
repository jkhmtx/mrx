use mrx_utils::Config;

use crate::{
    Options,
    build::build,
};

pub fn run(config: &Config, options: &Options) {
    if let Err(e) = build(config, options).map(|paths| {
        for p in paths {
            println!("{p}");
        }
    }) {
        eprintln!("{e}");

        std::process::exit(1);
    }
}
