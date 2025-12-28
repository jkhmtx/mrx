mod watch_files;

mod cli;
mod graph;

pub use cli::Options;
use cli::Target;
use mrx_utils::Config;

pub fn show(config: &Config, options: &Options) {
    match &options.target {
        Target::WatchFiles => watch_files::watch_files(config),
        Target::Graph(graph) => graph::graph(config, graph),
    }
}
