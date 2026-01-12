use mrx_utils::Config;

use crate::{
    Options,
    cli::Target,
    graph,
    watch_files,
};

pub(crate) fn show(config: &Config, options: &Options) {
    match options.target {
        Target::WatchFiles => watch_files::watch_files(config),
        Target::Graph(graph) => graph::graph(config, graph),
    }
}
