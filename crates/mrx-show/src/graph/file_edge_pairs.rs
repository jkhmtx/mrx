use std::path::Path;

use exn::ResultExt as _;
use mrx_utils::{
    Config,
    graph::{
        Edge,
        Graph,
        GraphNode,
    },
};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub(crate) enum FileEdgePairsError {
    #[error("WatchFilesError::CreatingGraph")]
    CreatingGraph,
    #[error("FileEdgePairsError::CalculatingRelativePath")]
    CalculatingRelativePath,
}

type FileEdgePairsResult<T> = Result<T, exn::Exn<FileEdgePairsError>>;

fn display(node: &GraphNode, dir: &Path) -> FileEdgePairsResult<String> {
    Ok(node
        .as_path()
        .as_relative_to_parent(dir)
        .or_raise(|| FileEdgePairsError::CalculatingRelativePath)?
        .to_string_lossy()
        .to_string())
}

pub(super) fn file_edge_pairs(config: &Config) -> FileEdgePairsResult<Vec<(String, String)>> {
    let graph = Graph::new(config).map_err(|e| e.raise(FileEdgePairsError::CreatingGraph))?;

    let dir = config.dir();
    graph
        .to_edges()
        .into_iter()
        .map(|Edge(a, b)| (display(&a, &dir), display(&b, &dir)))
        .map(|(a, b)| a.and_then(|a| b.map(|b| (a, b))))
        .collect::<Result<Vec<_>, _>>()
}
