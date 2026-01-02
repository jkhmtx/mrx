use mrx_utils::Config;

mod file_edge_pairs;

use crate::cli::{
    GraphFormat,
    GraphKind,
    GraphOptions,
};

pub(crate) fn graph(config: &Config, options: GraphOptions) {
    match options {
        GraphOptions {
            format: GraphFormat::EdgePairs,
            kind: GraphKind::Files,
        } => {
            let mut pairs = file_edge_pairs::file_edge_pairs(config);

            pairs.sort_by(|(a1, b1), (a2, b2)| match a1.cmp(a2) {
                std::cmp::Ordering::Equal => b1.cmp(b2),
                ord => ord,
            });

            for (a, b) in pairs {
                println!("{a} {b}");
            }
        }
        _ => todo!(),
    }
}
