use mrx_utils::{find_bin_attrnames, Config};

mod cli;

pub use cli::Options;

pub fn hook(config: &Config, _options: &Options) {
    let bins = {
        let mut bins = find_bin_attrnames(config);

        bins.sort();
        bins
    };

    println!("The following commands are available in your shell:");
    for bin in bins {
        println!("  {bin}");
    }
}
