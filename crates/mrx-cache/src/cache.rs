use std::{
    fs::File,
    os::unix::fs::MetadataExt,
    path::Path,
};

use mrx_utils::{
    Attrname,
    Config,
    fs::AbsolutePathBuf,
    graph::{
        Graph,
        GraphNode,
        NodeId,
    },
    nix_build_command::{
        NixBuildCommand,
        NixBuildError,
    },
    nix_references_command::{
        NixReferencesCommand,
        NixReferencesError,
    },
    nix_store_path::{
        MrxNixStorePath,
        NixStorePath,
    },
};
use thiserror::Error;

use crate::{
    DbError,
    Options,
    WriteStoreError,
    get_mtime,
    get_store_bin_path,
    set_alias_mtime,
    set_node_mtime,
    unix_seconds::UnixSeconds,
    write_store,
};

#[derive(Debug, Error)]
pub(crate) enum CacheError {
    #[error("No derivations provided. Provide at least one as a positional argument.")]
    NoDerivations,
    #[error("No fallback entrypoint 'flake.nix' or 'default.nix' found")]
    NoEntrypoint,
    #[error(transparent)]
    Build(#[from] NixBuildError),
    #[error(transparent)]
    References(#[from] NixReferencesError),
    #[error("Failed to read store: {0}")]
    ReadStore(DbError),
    #[error("Failed to write store: {0}")]
    WriteStore(WriteStoreError),
    #[error("TODO: {0}")]
    Todo(&'static str),
}

type CacheResult<T> = Result<T, CacheError>;

enum BuildStrategy {
    UseCached(Vec<NixStorePath>),
    Build(Vec<Attrname>),
}

impl BuildStrategy {
    fn new(attrnames: &[Attrname], stale: &[StaleNodeInfo]) -> Result<Self, CacheError> {
        let binpaths = if stale.is_empty() {
            Some(
                attrnames
                    .iter()
                    .filter_map(|attrname| {
                        get_store_bin_path(attrname)
                            .map_err(|e| {
                                eprintln!("{e}");
                                CacheError::Todo("get store bin paths")
                            })
                            .transpose()
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
        } else {
            None
        };

        Ok(match binpaths {
            None => {
                let stale_attrname_idxs = {
                    let mut stale_attrname_idxs =
                        stale.iter().map(|(idx, _, _)| *idx).collect::<Vec<_>>();

                    stale_attrname_idxs.dedup();

                    stale_attrname_idxs
                };

                Self::Build(
                    stale_attrname_idxs
                        .into_iter()
                        .filter_map(|idx| attrnames.get(idx))
                        .cloned()
                        .collect::<Vec<_>>(),
                )
            }
            Some(paths) if !paths.is_empty() => Self::UseCached(paths),
            _ => Self::Build(attrnames.to_vec()),
        })
    }
}

/// # Errors
/// TODO
/// # Panics
/// TODO
pub(crate) fn cache(config: &Config, options: &Options) -> CacheResult<Vec<NixStorePath>> {
    if options.derivations.is_empty() {
        return Err(CacheError::NoDerivations);
    }

    let graph = Graph::new(config).map_err(|e| {
        eprintln!("{e}");
        CacheError::Todo("new graph")
    })?;

    let attrnames = options
        .derivations
        .iter()
        .cloned()
        .map(Attrname)
        .collect::<Vec<_>>();

    let stale = find_stale_node_infos(config, &graph, &attrnames).map_err(|e| {
        eprintln!("{e}");
        CacheError::ReadStore(e)
    })?;

    for (_, node, file_mtime) in &stale {
        set_mtime(node, *file_mtime).unwrap();
    }

    let to_build = match BuildStrategy::new(&attrnames, &stale)? {
        BuildStrategy::UseCached(paths) => return Ok(paths),
        BuildStrategy::Build(attrnames) => attrnames
            .iter()
            .map(|name| format!("#{name}"))
            .collect::<Vec<_>>(),
    };

    eprintln!("Rebuilding {}", &to_build.join(" "));

    let build_command = config
        .get_entrypoint()
        .map(|entrypoint| NixBuildCommand::new(entrypoint, &to_build))
        .ok_or(CacheError::NoEntrypoint)?;

    let out_paths = build_command
        .execute()?
        .into_iter()
        .filter_map(|output| output.out)
        .collect::<Vec<_>>();

    let reference_paths = NixReferencesCommand::new(out_paths.as_slice())
        .execute()?
        .store_paths
        .into_iter()
        .filter_map(|path| match path {
            NixStorePath::MrxOutDir(MrxNixStorePath(path, ref attrname)) => Some((
                NixStorePath::new(path + "/bin/" + attrname),
                attrname.clone(),
            )),
            NixStorePath::MrxBinDir(MrxNixStorePath(path, ref attrname)) => {
                Some((NixStorePath::new(path + attrname), attrname.clone()))
            }
            NixStorePath::MrxExe(MrxNixStorePath(path, ref attrname)) => {
                Some((NixStorePath::new(path), attrname.clone()))
            }
            _ => None,
        });

    for (path, attrname) in reference_paths {
        // First of two attempts to write the store path
        let write_store_result = write_store(&attrname, &path);

        if matches!(write_store_result, Err(WriteStoreError::MissingAlias)) {
            // If we failed to write the store, set the alias and try again
            if let Some((_, graph_node)) = graph.find_node(&NodeId::Attrname(attrname.clone())) {
                let file_mtime = get_file_mtime(&graph_node.path);

                set_alias_mtime(&attrname, &graph_node.path, file_mtime)
                    .map_err(WriteStoreError::DbError)
                    .and_then(|()| write_store(&attrname, &path))
            } else {
                Ok(())
            }
        } else {
            write_store_result
        }
        .map_err(|e| {
            eprintln!("{e}");
            CacheError::WriteStore(e)
        })?;
    }

    Ok(out_paths
        .into_iter()
        .filter_map(NixStorePath::into_mrx_exe)
        .collect())
}

type StaleNodeInfo<'a> = (usize, &'a GraphNode, UnixSeconds);

fn find_stale_node_infos<'a>(
    config: &Config,
    graph: &'a Graph,
    attrnames: &'a [Attrname],
) -> Result<Vec<StaleNodeInfo<'a>>, DbError> {
    let mut stale_nodes = vec![];
    for (attrname_idx, node) in attrnames
        .iter()
        .cloned()
        .map(NodeId::Attrname)
        .enumerate()
        .filter_map(|(attrname_idx, id)| graph.find_node(&id).map(|(_, node)| (attrname_idx, node)))
    {
        if let Some(file_mtime) = is_stale(node)? {
            stale_nodes.push((attrname_idx, node, file_mtime));
        } else {
            let dependencies = graph.find_dependencies_of(attrname_idx);

            let generated_out_path =
                AbsolutePathBuf::try_from(config.get_generated_out_path().as_path())
                    .expect("generated out path must be resolvable");

            for node in dependencies
                .values()
                .filter(|node| node.path != generated_out_path)
                .map(|node| {
                    node.derivation.as_ref().map_or_else(
                        || NodeId::Path(node.path.clone()),
                        |drv| NodeId::Attrname(Attrname(drv.to_string())),
                    )
                })
                .filter_map(|id| graph.find_node(&id))
                .map(|(_, node)| node)
            {
                if let Some(file_mtime) = is_stale(node)? {
                    stale_nodes.push((attrname_idx, node, file_mtime));
                }
            }
        }
    }

    Ok(stale_nodes)
}

fn get_file_mtime(path: impl AsRef<Path>) -> UnixSeconds {
    File::open(path)
        .ok()
        .and_then(|file| {
            file.metadata()
                .ok()
                .map(|metadata| metadata.mtime())
                .map(UnixSeconds::from)
        })
        .unwrap_or_default()
}

fn is_stale(node: &GraphNode) -> Result<Option<UnixSeconds>, DbError> {
    let file_mtime = get_file_mtime(&node.path);

    let node_id = NodeId::Path(node.path.clone());

    if get_mtime(&node_id)?.is_none_or(|saved_mtime| saved_mtime < file_mtime) {
        Ok(Some(file_mtime))
    } else {
        Ok(None)
    }
}

fn set_mtime(node: &GraphNode, mtime: UnixSeconds) -> Result<(), DbError> {
    if let Some(attrname) = &node.derivation {
        set_alias_mtime(attrname, &node.path, mtime)
    } else {
        set_node_mtime(&node.path, mtime).map(|_| {})
    }
}
