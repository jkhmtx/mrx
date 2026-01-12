mod cli;

use mrx_utils::Config;

use crate::cli::{
    Mrx,
    MrxCommand,
    Plumbing,
};

#[tokio::main]
async fn main() {
    let (config, options) = Mrx::args().unwrap();
    handle(config, options).await;
}

async fn handle(config: Config, options: Mrx) {
    match options.command {
        MrxCommand::Build(opts) => mrx_build::run(&config, &opts),
        MrxCommand::Plumbing(opts) => match opts {
            Plumbing::Cache(opts) => mrx_cache::run(&config, &opts).await,
        },
        MrxCommand::Generate(opts) => mrx_generate::run(&config, &opts),
        MrxCommand::Hook(opts) => mrx_hook::run(&config, &opts),
        MrxCommand::Show(opts) => mrx_show::run(&config, &opts),
    };
}
