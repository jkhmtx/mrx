use clap::Parser;
use mrx_utils::{mrx_cli, MrxCli};

/// Build the project according to the manifest
#[mrx_cli]
#[derive(Parser, MrxCli, Debug)]
pub struct Options {
    /// After building, cache the build results into an out-of-store directory
    #[arg(long, default_value_t = true)]
    pub cache: bool,
}
