use mrx_utils::Config;

use crate::{
    Options,
    show::show,
};

pub fn run(config: &Config, options: &Options) {
    show(config, options);
}
