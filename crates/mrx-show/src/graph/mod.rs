use mrx_utils::Config;

mod file_edge_pairs;

use thiserror::Error as ThisError;

use crate::cli::{
    GraphFormat,
    GraphKind,
    GraphOptions,
};

#[derive(Debug, ThisError)]
pub(crate) enum ShowGraphError {
    #[error("ShowGraphError::FileEdges")]
    FileEdges,
}

type ShowGraphResult<T> = Result<T, exn::Exn<ShowGraphError>>;

pub(crate) fn graph(config: &Config, options: GraphOptions) -> ShowGraphResult<()> {
    match options {
        GraphOptions {
            format: GraphFormat::EdgePairs,
            kind: GraphKind::Files,
        } => {
            let mut pairs = file_edge_pairs::file_edge_pairs(config)
                .map_err(|e| e.raise(ShowGraphError::FileEdges))?;

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

    Ok(())
}
