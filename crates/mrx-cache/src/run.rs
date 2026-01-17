use mrx_utils::{
    Config,
    nix_store_path::NixStorePath,
};

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
        Ok(paths) => {
            for string in paths.into_iter().map(NixStorePath::into_string) {
                println!("{string}");
            }
        }
    }
}
