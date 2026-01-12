use mrx_utils::Config;

use crate::{
    Options,
    cache::cache,
};

/// ### Panics
/// TODO
pub async fn run(config: &Config, options: &Options) {
    match cache(config, options).await {
        Err(e) => {
            eprintln!("{e}");

            std::process::exit(1);
        }
        Ok(strings) => {
            for string in strings {
                println!("{string}");
            }
        }
    }
}
