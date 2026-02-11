use std::{
    fs::File,
    os::unix::fs::MetadataExt,
    path::Path,
};

use chrono::{
    DateTime,
    Utc,
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
    time::Time,
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
    WriteStore(DbError),
    #[error("TODO: {0}")]
    Todo(&'static str),
}

type CacheResult<T> = Result<T, CacheError>;

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

    if stale.is_empty() {
        let binpaths = attrnames
            .iter()
            .filter_map(|attrname| {
                get_store_bin_path(attrname)
                    .map_err(|_| CacheError::Todo("get store bin paths"))
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;

        debug_assert!(!binpaths.is_empty());

        return Ok(binpaths);
    }

    for (_, node, file_mtime) in &stale {
        set_mtime(node, file_mtime).unwrap();
    }

    let derivation_build_strings = find_derivation_build_strings(&attrnames, &stale);

    eprintln!("Rebuilding {}", &derivation_build_strings.join(" "));

    let build_command = config
        .get_entrypoint()
        .map(|entrypoint| NixBuildCommand::new(entrypoint, &derivation_build_strings))
        .ok_or(CacheError::NoEntrypoint)?;

    let out_paths = build_command
        .execute()?
        .into_iter()
        .filter_map(|output| output.out)
        .collect::<Vec<_>>();

    let reference_paths = NixReferencesCommand::new(out_paths.as_slice())
        .execute()?
        .store_paths;

    for path in reference_paths {
        if let Some((path, attrname)) = match path {
            NixStorePath::MrxOutDir(MrxNixStorePath(path, ref attrname)) => {
                Some((path + "/bin/" + attrname, attrname.clone()))
            }
            NixStorePath::MrxBinDir(MrxNixStorePath(path, ref attrname)) => {
                Some((path + attrname, attrname.clone()))
            }
            NixStorePath::MrxExe(MrxNixStorePath(path, ref attrname)) => {
                Some((path, attrname.clone()))
            }
            _ => None,
        } {
            match write_store(&attrname, NixStorePath::new(path.clone())) {
                Ok(()) => Ok(()),
                Err(WriteStoreError::MissingAlias) => {
                    if let Some((_, graph_node)) =
                        graph.find_node(&NodeId::Attrname(attrname.clone()))
                    {
                        let file_mtime = get_file_mtime(&graph_node.path);

                        set_alias_mtime(&attrname, &graph_node.path, &file_mtime).and_then(|()| {
                            match write_store(&attrname, NixStorePath::new(path)) {
                                Ok(()) => Ok(()),
                                Err(WriteStoreError::MissingAlias) => {
                                    unreachable!("Failed for missing alias after inserting alias")
                                }
                                Err(WriteStoreError::DbError(e)) => Err(e),
                            }
                        })
                    } else {
                        Ok(())
                    }
                }
                Err(WriteStoreError::DbError(e)) => Err(e),
            }
            .map_err(|e| {
                eprintln!("{e}");
                CacheError::WriteStore(e)
            })?;
        }
    }

    Ok(out_paths
        .into_iter()
        .filter_map(NixStorePath::into_mrx_exe)
        .collect())
}

type StaleNodeInfo<'a> = (usize, &'a GraphNode, DateTime<Utc>);

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

fn get_file_mtime(path: impl AsRef<Path>) -> Time {
    File::open(path)
        .ok()
        .and_then(|file| {
            file.metadata()
                .ok()
                .map(|metadata| metadata.mtime())
                .and_then(Time::from_timestamp_secs)
        })
        .unwrap_or_default()
}

fn is_stale(node: &GraphNode) -> Result<Option<DateTime<Utc>>, DbError> {
    let file_mtime = get_file_mtime(&node.path);

    let node_id = NodeId::Path(node.path.clone());

    if get_mtime(node_id)?.is_none_or(|saved_mtime| saved_mtime < file_mtime) {
        Ok(Some(file_mtime))
    } else {
        Ok(None)
    }
}

fn set_mtime(node: &GraphNode, mtime: &DateTime<Utc>) -> Result<(), DbError> {
    if let Some(attrname) = &node.derivation {
        set_alias_mtime(attrname, &node.path, mtime)
    } else {
        set_node_mtime(&node.path, mtime).map(|_| {})
    }
}

fn find_derivation_build_strings(attrnames: &[Attrname], stale: &[StaleNodeInfo]) -> Vec<String> {
    let stale_attrname_idxs = {
        let mut stale_attrname_idxs = stale.iter().map(|(idx, _, _)| *idx).collect::<Vec<_>>();

        stale_attrname_idxs.dedup();

        stale_attrname_idxs
    };

    stale_attrname_idxs
        .into_iter()
        .filter_map(|idx| attrnames.get(idx))
        .map(|attrname| format!("#{attrname}"))
        .collect::<Vec<_>>()
}
