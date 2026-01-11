use std::{
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
    set_alias_mtime,
    set_node_mtime,
    time::Time,
};

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("No derivations provided. Provide at least one as a positional argument.")]
    NoDerivations,
    #[error("No fallback entrypoint 'flake.nix' or 'default.nix' found")]
    NoEntrypoint,
    #[error(transparent)]
    Build(#[from] NixBuildError),
    #[error("{0}: {1} (Path: {2:?})")]
    Io(&'static str, std::io::Error, Option<PathBuf>),
    #[error("TODO")]
    Todo,
}

type CacheResult<T> = Result<T, CacheError>;

/// # Errors
/// TODO
/// # Panics
/// TODO
pub async fn cache(config: &Config, options: &Options) -> CacheResult<()> {
    if options.derivations.is_empty() {
        return Err(CacheError::NoDerivations);
    }

    let graph = Graph::new(config).or(Err(CacheError::Todo))?;

    let mut derivations = vec![];

    for derivation in &options.derivations {
        let id = NodeId::Attrname(Attrname(derivation.clone()));
        if is_stale(&graph, id).await {
            dbg!(derivation);
            derivations.push(format!("#{derivation}"));
        }
    }

    if derivations.is_empty() {
        return Ok(());
    }

    eprintln!("Rebuilding {}", &derivations.join(" "));

    let build_command = config
        .get_entrypoint()
        .map(|entrypoint| NixBuildCommand::new(entrypoint, &derivations))
        .ok_or(CacheError::NoEntrypoint)?;

    let cache_dir = {
        let dir = config.state_dir();

        dir.join("cache")
    };
    mrx_utils::fs::mk_dir(&cache_dir)
        .map_err(|e| CacheError::Io("Failed to make directory:", e, Some(cache_dir.clone())))?;

    for path in build_command
        .execute()?
        .into_iter()
        .filter_map(|output| output.out)
    {
        let derivation = path
            .split_once('-')
            .map(|(_, derivation)| derivation)
            .expect("derivation outpath should follow the form '/nix/store/123abc-[derivation]'");

        let path = PathBuf::from(&path);

        std::fs::remove_file(cache_dir.join(derivation)).or_else(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(CacheError::Io(
                    "Failed to remove file:",
                    e,
                    Some(cache_dir.join(derivation).clone()),
                ))
            }
        })?;

        std::os::unix::fs::symlink(
            path.join("bin").join(derivation),
            cache_dir.join(derivation),
        )
        .map_err(|e| CacheError::Io("Failed to symlink:", e, None))?;
    }

    Ok(())
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
