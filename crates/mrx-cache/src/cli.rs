use clap::Parser;
use mrx_utils::{mrx_cli, MrxCli};

/// Build and symlink derivations
#[mrx_cli]
#[derive(Parser, MrxCli)]
pub struct Options {
    /// The derivations to cache
    pub derivations: Vec<String>,
}
