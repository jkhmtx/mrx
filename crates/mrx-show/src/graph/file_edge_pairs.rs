use std::path::Path;

use mrx_utils::{
    Config,
    graph::{
        Edge,
        Graph,
        GraphNode,
    },
};

fn display(node: &GraphNode, dir: &Path) -> String {
    node.as_path()
        .as_relative_to_parent(dir)
        .unwrap()
        .to_string_lossy()
        .to_string()
}

pub(super) fn file_edge_pairs(config: &Config) -> Vec<(String, String)> {
    let graph = Graph::new(config).unwrap();

    let dir = config.dir();
    graph
        .to_edges()
        .into_iter()
        .map(|Edge(a, b)| (display(&a, &dir), display(&b, &dir)))
        .collect::<Vec<_>>()
}
