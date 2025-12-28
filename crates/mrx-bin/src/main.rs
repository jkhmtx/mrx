mod cli;

use mrx_build::build;
use mrx_cache::cache;
use mrx_generate::generate;
use mrx_hook::hook;
use mrx_refresh::refresh;
use mrx_show::show;
use mrx_utils::Config;

use crate::cli::{
    Mrx,
    MrxCommand,
    Plumbing,
};

fn main() {
    let (config, options) = Mrx::args().unwrap();

    if let Err(e) = handle(config, options) {
        eprintln!("{}", e);

        std::process::exit(1);
    }
}

fn handle(config: Config, options: Mrx) -> anyhow::Result<()> {
    match options.command {
        MrxCommand::Build(opts) => build(&config, &opts).map(|paths| {
            for p in paths.into_iter() {
                println!("{}", p);
            }
        })?,
        MrxCommand::Plumbing(opts) => match opts {
            Plumbing::Cache(opts) => cache(&config, &opts)?,
        },
        MrxCommand::Generate(opts) => generate(&config, &opts)?,
        MrxCommand::Hook(opts) => hook(&config, &opts),
        MrxCommand::Refresh(opts) => refresh(&config, &opts),
        MrxCommand::Show(opts) => show(&config, &opts),
    };

    Ok(())
}
