use exn::ResultExt;
use mrx_utils::{
    Attrname,
    Config,
    fs::AbsolutePathBuf,
    graph::{
        Graph,
        NodeId,
    },
};
use thiserror::Error as ThisError;

use crate::cli::WatchFilesOptions;

#[derive(Debug, ThisError)]
pub(crate) enum ShowWatchFilesError {
    #[error("WatchFilesError::CreatingGraph")]
    CreatingGraph,
    #[error("WatchFilesError::GettingGeneratedPaths")]
    GettingGeneratedPaths,
    #[error("WatchFilesError::InvalidPaths")]
    InvalidPaths,
}

type ShowWatchFilesResult<T> = Result<T, exn::Exn<ShowWatchFilesError>>;

pub(crate) fn watch_files(
    config: &Config,
    options: &WatchFilesOptions,
) -> ShowWatchFilesResult<Vec<String>> {
    let graph = Graph::new(config).map_err(|e| e.raise(ShowWatchFilesError::CreatingGraph))?;
    let generated_out_path = AbsolutePathBuf::try_from(config.get_generated_out_path().as_path())
        .or_raise(|| ShowWatchFilesError::GettingGeneratedPaths)?;

    {
        let mut files = if options.derivations.is_empty() {
            graph.to_nodes()
        } else {
            let mut files: Vec<&AbsolutePathBuf> = vec![];

            for (idx, node) in options.derivations.iter().filter_map(|derivation| {
                Attrname::try_from(derivation.as_str())
                    .map(NodeId::Attrname)
                    .ok()
                    .and_then(|id| graph.find_node(&id))
            }) {
                files.push(&node.path);

                files.extend(
                    graph
                        .find_dependencies_of(idx)
                        .values()
                        .map(|node| &node.path),
                );
            }

            files
        }
        .into_iter()
        .filter(|path| **path != generated_out_path)
        .map(|path| {
            path.as_relative_to_parent(&config.dir())
                .map(|path| path.to_string_lossy().to_string())
        })
        .collect::<Result<Vec<_>, _>>()
        .or_raise(|| ShowWatchFilesError::InvalidPaths)?;

        files.dedup();
        files.sort();

        Ok(files)
    }
}
