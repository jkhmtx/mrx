use std::{
    fs::File,
    os::unix::fs::MetadataExt,
    path::Path,
};

use exn::{
    ResultExt,
    bail,
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
    nix_build_command::NixBuildCommand,
    nix_references_command::NixReferencesCommand,
    nix_store_path::{
        MrxNixStorePath,
        NixStorePath,
    },
};
use thiserror::Error as ThisError;

use crate::{
    Options,
    get_mtime,
    get_store_bin_path,
    set_alias_mtime,
    set_node_mtime,
    unix_seconds::UnixSeconds,
    write_store,
};

#[derive(Debug, ThisError)]
pub(crate) enum CacheError {
    #[error("CacheError::Static: {0}")]
    Static(&'static str),
    #[error("CacheError::Build: {0}")]
    Build(String),
    #[error("CacheError::StoreNode: {0}: {1}")]
    StoreNode(GraphNode, &'static str),
    #[error("CacheError::WriteStore")]
    WriteStore,
}

type CacheResult<T> = Result<T, exn::Exn<CacheError>>;

enum BuildStrategy {
    UseCached(Vec<NixStorePath>),
    Build(Vec<Attrname>),
}

impl BuildStrategy {
    fn new(attrnames: &[Attrname], stale: &[StaleNodeInfo]) -> CacheResult<Self> {
        let binpaths = if stale.is_empty() {
            Some(
                attrnames
                    .iter()
                    .filter_map(|attrname| {
                        get_store_bin_path(attrname)
                            .map_err(|e| {
                                e.raise(CacheError::Build(format!(
                                    "failed to get store bin paths for attrname: '{attrname}'"
                                )))
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
        bail!(CacheError::Static(
            "No derivations provided. Provide at least one as a positional argument."
        ));
    }

    let graph =
        Graph::new(config).or_raise(|| CacheError::Static("Failed to create dependency graph"))?;
    // .map_err(|e| e.raise(CacheError::Static("Failed to create dependency graph")))?;

    let attrnames = options
        .derivations
        .iter()
        .cloned()
        .map(Attrname)
        .collect::<Vec<_>>();

    let stale = find_stale_node_infos(config, &graph, &attrnames)?;

    for (_, node, file_mtime) in &stale {
        set_mtime(node, *file_mtime)?;
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
        .ok_or(CacheError::Static(
            "No fallback entrypoint 'flake.nix' or 'default.nix' found",
        ))?;

    let out_paths = build_command
        .execute()
        .map_err(|e| e.raise(CacheError::Static("Build command failed")))?
        .into_iter()
        .filter_map(|output| output.out)
        .collect::<Vec<_>>();

    let reference_paths = NixReferencesCommand::new(out_paths.as_slice())
        .execute()
        .map_err(|e| e.raise(CacheError::Static("References command failed")))?
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
        // (1/2) attempts to write the store path
        match write_store(&attrname, &path) {
            Ok(()) => Ok(()),
            Err(e) if e.is_missing_alias() => {
                // final attempt
                let (_, graph_node) = graph
                    .find_node(&NodeId::Attrname(attrname.clone()))
                    .expect("attrname must exist in graph");

                let file_mtime = get_file_mtime(&graph_node.path);

                set_alias_mtime(&attrname, &graph_node.path, file_mtime)
                    .map_err(|e| e.raise(CacheError::WriteStore))?;

                write_store(&attrname, &path)
            }
            Err(e) => Err(e),
        }
        .map_err(|e| e.raise(CacheError::WriteStore))?;
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
) -> CacheResult<Vec<StaleNodeInfo<'a>>> {
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

fn is_stale(node: &GraphNode) -> CacheResult<Option<UnixSeconds>> {
    let file_mtime = get_file_mtime(&node.path);

    let node_id = NodeId::Path(node.path.clone());

    Ok(
        if get_mtime(&node_id)
            .map_err(|e| {
                e.raise(CacheError::StoreNode(
                    node.clone(),
                    "failed to get mtime for node",
                ))
            })?
            .is_none_or(|saved_mtime| saved_mtime < file_mtime)
        {
            Some(file_mtime)
        } else {
            None
        },
    )
}

fn set_mtime(node: &GraphNode, mtime: UnixSeconds) -> CacheResult<()> {
    if let Some(attrname) = &node.derivation {
        set_alias_mtime(attrname, &node.path, mtime)
    } else {
        set_node_mtime(&node.path, mtime).map(|_| {})
    }
    .or_raise(|| CacheError::StoreNode(node.clone(), "failed to set mtime for node"))
}
