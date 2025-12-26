use clap::{Parser, Subcommand};
use mrx_build::Options as BuildOptions;
use mrx_cache::Options as CacheOptions;
use mrx_generate::Options as GenerateOptions;
use mrx_hook::Options as HookOptions;
use mrx_refresh::Options as RefreshOptions;
use mrx_show::Options as ShowOptions;
use mrx_utils::{mrx_cli, MrxCli};

/// Commands considered "plumbing" which are not generally intended for end users.
#[derive(Subcommand)]
pub enum Plumbing {
    Cache(CacheOptions),
}

#[derive(Subcommand)]
pub enum MrxCommand {
    Build(BuildOptions),
    Generate(GenerateOptions),
    Hook(HookOptions),
    Refresh(RefreshOptions),
    Show(ShowOptions),
    #[command(subcommand)]
    Plumbing(Plumbing),
}

#[mrx_cli]
#[derive(Parser, MrxCli)]
#[command(version, about)]
pub struct Mrx {
    #[command(subcommand)]
    pub command: MrxCommand,
}
