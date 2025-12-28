use clap::Parser;
use mrx_utils::{mrx_cli, MrxCli};

/// Refresh the build cache if it exists
#[mrx_cli]
#[derive(Parser, MrxCli, Debug)]
pub struct Options {}
