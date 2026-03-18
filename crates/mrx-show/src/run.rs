use mrx_utils::Config;

use crate::{
    Options,
    show::show,
};

/// # Panics
/// TODO
pub fn run(config: &Config, options: &Options) {
    show(config, options).unwrap();
}
