use mrx_utils::{
    graph::{Edge, Graph},
    Config,
};

pub fn file_edge_pairs(config: &Config) {
    let graph = Graph::try_from(config.get_entrypoint().unwrap()).unwrap();

    let edges = graph.to_edges();

    for Edge(a, b) in edges {
        println!(
            "{} {}",
            a.as_path().as_path().display(),
            b.as_path().as_path().display()
        );
    }
}
