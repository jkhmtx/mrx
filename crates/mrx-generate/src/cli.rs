use clap::Parser;
use mrx_utils::{
    MrxCli,
    mrx_cli,
};

/// Generate the project's barrel file
#[mrx_cli]
#[derive(Parser, MrxCli, Debug)]
pub struct Options {}
