use clap::Parser;

use mrx_utils::{mrx_cli, MrxCli};

/// Generate the project's barrel file
#[mrx_cli]
#[derive(Parser, MrxCli)]
pub struct Options {}
