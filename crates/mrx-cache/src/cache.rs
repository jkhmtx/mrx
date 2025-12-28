use std::path::PathBuf;

use mrx_utils::{
    Config,
    nix_build_command::{
        NixBuildCommand,
        NixBuildError,
    },
};
use thiserror::Error;

use crate::Options;

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
pub fn cache(config: &Config, options: &Options) -> CacheResult<()> {
    let derivations = options
        .derivations
        .iter()
        .map(|drv| format!("#{drv}"))
        .collect::<Vec<_>>();

    if derivations.is_empty() {
        return Err(CacheError::NoDerivations);
    }

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
