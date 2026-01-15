use clap::Parser;
use mrx_utils::{
    MrxCli,
    mrx_cli,
};

/// Build the project according to the manifest
#[mrx_cli]
#[derive(Parser, MrxCli, Debug)]
pub struct Options {
    /// Use 'mrx generate' before building.
    #[arg(long, default_value_t = false)]
    pub generate: bool,

    /// Use 'mrx hook' after building. If '--skip-bin' is also specified, this option is ignored and no hook will be shown.
    #[arg(long, default_value_t = false)]
    pub hook: bool,

    /// Normally, after building, this command prepares the local binary directory and echoes its path. When this directory is in PATH, they become available as '_.{name}'.
    ///
    /// When this flag is set, only the build command will be run: 'nix build #{installables}'.
    #[arg(long, default_value_t = false)]
    pub skip_bin: bool,
}
