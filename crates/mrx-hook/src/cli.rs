use clap::Parser;
use mrx_utils::{mrx_cli, MrxCli};

/// Print the post-build shell hook
#[mrx_cli]
#[derive(Parser, MrxCli, Debug)]
pub struct Options {}
