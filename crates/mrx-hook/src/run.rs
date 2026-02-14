use mrx_utils::Config;

use crate::{
    Options,
    hook::hook,
};

pub fn run(config: &Config, options: &Options) {
    let bins = hook(config, options);

    eprintln!("The following commands are available in your shell:");
    for bin in bins {
        eprintln!("  - {bin}");
    }
}
