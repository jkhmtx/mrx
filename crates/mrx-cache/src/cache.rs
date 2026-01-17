use std::{
    fs::File,
    os::unix::fs::MetadataExt,
    path::Path,
};

use mrx_utils::{
    Attrname,
    Config,
    graph::{
        Graph,
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
    Options,
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
    #[error("TODO: {0}")]
    Todo(&'static str),
}

type CacheResult<T> = Result<T, CacheError>;

/// # Errors
/// TODO
/// # Panics
/// TODO
pub(crate) async fn cache(config: &Config, options: &Options) -> CacheResult<Vec<NixStorePath>> {
    if options.derivations.is_empty() {
        return Err(CacheError::NoDerivations);
    }

    let graph = Graph::new(config).map_err(|e| {
        dbg!(e);
        CacheError::Todo("new graph")
    })?;

    let attrnames = options
        .derivations
        .iter()
        .cloned()
        .map(Attrname)
        .collect::<Vec<_>>();

    let mut stale = vec![];

    for attrname in &attrnames {
        let id = NodeId::Attrname(attrname.clone());
        if is_stale(&graph, id).await {
            stale.push(attrname);
        }
    }

    if stale.is_empty() {
        let mut binpaths = vec![];
        for attrname in &attrnames {
            if let Some(path) = get_store_bin_path(attrname)
                .await
                .map_err(|_| CacheError::Todo("get store bin paths"))?
            {
                binpaths.push(path);
            }
        }

        debug_assert!(!binpaths.is_empty());

        return Ok(binpaths);
    }

    let derivation_build_strings = stale
        .iter()
        .map(|attrname| format!("#{attrname}"))
        .collect::<Vec<_>>();

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
            write_store(attrname, NixStorePath::new(path))
                .await
                .map_err(|_| CacheError::Todo("mrx exe"))?;
        }
    }

    Ok(out_paths
        .into_iter()
        .filter_map(NixStorePath::into_mrx_exe)
        .collect())
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

async fn is_stale(graph: &Graph, id: NodeId) -> bool {
    if let Some((idx, node)) = graph.find_node(&id) {
        let mtime = get_mtime(NodeId::Path(node.path.clone()))
            .await
            .ok()
            .flatten();

        let file_mtime = get_file_mtime(&node.path);
        let stale = mtime.is_none_or(|mtime| file_mtime > mtime);

        if stale {
            if let Some(attrname) = &node.derivation {
                set_alias_mtime(attrname, &node.path, &file_mtime)
                    .await
                    .unwrap();
            } else {
                let _ = set_node_mtime(&node.path, &file_mtime).await.unwrap();
            }
        }

        let dependencies = graph.find_dependencies_of(idx);

        let ids = dependencies.values().map(|node| {
            node.derivation.as_ref().map_or_else(
                || NodeId::Path(node.path.clone()),
                |drv| NodeId::Attrname(Attrname(drv.to_string())),
            )
        });

        let mut has_stale_children = false;

        for id in ids {
            let stale = Box::pin(is_stale(graph, id)).await;

            has_stale_children = has_stale_children || stale;
        }

        stale || has_stale_children
    } else {
        false
    }
}
