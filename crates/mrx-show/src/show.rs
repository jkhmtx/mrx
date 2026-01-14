use mrx_utils::Config;

use crate::{
    Options,
    cli::Target,
    graph,
    watch_files,
};

pub(crate) fn show(config: &Config, options: &Options) {
    match &options.target {
        Target::WatchFiles(watch) => {
            let files = watch_files::watch_files(config, watch);

            for file in &files {
                println!("{file}");
            }
        }
        Target::Graph(graph) => graph::graph(config, *graph),
    }
}
