use exn::ResultExt;
use mrx_utils::Config;
use thiserror::Error as ThisError;

use crate::{
    Options,
    cli::Target,
    graph,
    watch_files,
};

#[derive(Debug, ThisError)]
pub(crate) enum ShowError {
    #[error("ShowError::WatchFiles")]
    WatchFiles,
    #[error("ShowError::Graph")]
    Graph,
}

type ShowResult<T> = Result<T, exn::Exn<ShowError>>;

pub(crate) fn show(config: &Config, options: &Options) -> ShowResult<()> {
    match &options.target {
        Target::WatchFiles(watch) => {
            let files =
                watch_files::watch_files(config, watch).or_raise(|| ShowError::WatchFiles)?;

            for file in &files {
                println!("{file}");
            }
        }
        Target::Graph(graph) => {
            graph::graph(config, *graph).map_err(|e| e.raise(ShowError::Graph))?;
        }
    }

    Ok(())
}
