use clap::{Parser, Subcommand, ValueEnum};
use mrx_utils::{mrx_cli, MrxCli};

#[derive(ValueEnum, Clone, Default)]
pub enum GraphFormat {
    /// Display the edges of the graph, with start nodes on the left, end nodes on the right
    #[default]
    EdgePairs,
    /// Display the graph as a tree (helpful for quickly identifying cycles)
    Tree,
    /// Show unreachable nodes in the dependency graph that are 'lib'.'bin' and 'pkg' nodes are always considered reachable.
    UnreachableLib,
}

#[derive(ValueEnum, Clone, Default)]
pub enum GraphKind {
    /// Nodes will show as file paths
    #[default]
    Files,
    /// Nodes will show as mrx derivation handles (e.g. '_.foo.bar'), with non-handle nodes omitted
    Handles,
}

#[derive(Parser)]
pub struct GraphOptions {
    /// What format to use for the displayed graph
    #[arg(short, long, value_enum, default_value_t)]
    pub format: GraphFormat,
    /// What node representation to use
    #[arg(short, long, value_enum, default_value_t)]
    pub kind: GraphKind,
}

#[derive(Subcommand)]
pub enum Target {
    /// Watch files are nodes in the dependency graph.
    /// These files may be consumed by another program, such as 'direnv' or 'entr', to signal that "on files changed" work needs to be done.
    WatchFiles,
    /// Dependency graph of the build system
    Graph(GraphOptions),
}

/// Print some aspect of the mrx configuration or build system to stdout
#[mrx_cli]
#[derive(Parser, MrxCli)]
pub struct Options {
    /// What to show
    #[command(subcommand)]
    pub target: Target,
}
