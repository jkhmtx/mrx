use clap::Parser;
use mrx_utils::{
    MrxCli,
    mrx_cli,
};

/// Refresh the build cache if it exists
#[mrx_cli]
#[derive(Parser, MrxCli, Debug)]
pub struct Options {}
