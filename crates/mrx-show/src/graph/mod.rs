use mrx_utils::Config;

mod file_edge_pairs;

use crate::cli::{GraphFormat, GraphKind, GraphOptions};

pub fn graph(config: Config, options: GraphOptions) {
    match options {
        GraphOptions {
            format: GraphFormat::EdgePairs,
            kind: GraphKind::Files,
        } => file_edge_pairs::file_edge_pairs(config),
        _ => todo!(),
    }
}
