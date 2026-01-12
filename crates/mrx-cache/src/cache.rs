use std::{
    collections::HashMap,
    fs::File,
    os::unix::fs::MetadataExt,
    path::{
        Path,
        PathBuf,
    },
};

use mrx_utils::{
    Attrname,
    Config,
    fs::AbsoluteFilePathBuf,
    graph::{
        Graph,
        NodeId,
    },
    nix_build_command::{
        NixBuildCommand,
        NixBuildError,
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
    #[error("TODO")]
    Todo,
}

type CacheResult<T> = Result<T, CacheError>;

/// # Errors
/// TODO
/// # Panics
/// TODO
pub(crate) async fn cache(config: &Config, options: &Options) -> CacheResult<Vec<String>> {
    if options.derivations.is_empty() {
        return Err(CacheError::NoDerivations);
    }

    let graph = Graph::new(config).or(Err(CacheError::Todo))?;

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
                .map_err(|_| CacheError::Todo)?
            {
                binpaths.push(path);
            }
        }

        if !binpaths.is_empty() {
            return Ok(binpaths);
        }
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

    let bin_paths = build_command
        .execute()?
        .into_iter()
        .filter_map(|output| output.out)
        .map(|path| {
            let derivation = path
                .split_once('-')
                .map(|(_, derivation)| derivation)
                .expect(
                    "derivation outpath should follow the form '/nix/store/123abc-[derivation]'",
                );

            (
                Attrname(derivation.to_string()),
                PathBuf::from(&path).join("bin").join(derivation),
            )
        })
        .collect::<HashMap<_, _>>();

    for (alias, store_path) in &bin_paths {
        let store_path =
            AbsoluteFilePathBuf::try_from(store_path.as_path()).map_err(|_| CacheError::Todo)?;
        write_store(alias, &store_path)
            .await
            .map_err(|_| CacheError::Todo)?;
    }

    Ok(bin_paths
        .values()
        .map(|store_path| store_path.to_string_lossy().to_string())
        .collect())
}

fn get_file_mtime(path: &Path) -> Time {
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

        let file_mtime = get_file_mtime(node.path.as_path());
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
