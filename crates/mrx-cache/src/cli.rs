use clap::Parser;
use mrx_utils::{
    MrxCli,
    mrx_cli,
};

/// Build and run derivations, caching the result of the build for fast recall
#[mrx_cli]
#[derive(Parser, MrxCli, Debug)]
pub struct Options {
    /// The derivations to cache
    pub derivations: Vec<String>,
}
