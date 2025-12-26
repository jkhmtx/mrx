use crate::cli::Options;
use mrx_utils::fs::recreate_dir;
use mrx_utils::nix_build_command::{NixBuildCommand, NixBuildError, NixBuildOutput};
use mrx_utils::{find_bin_attrnames, Config};

use std::fmt::Write as _;
use std::os::unix::fs::PermissionsExt as UnixPermissions;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("No entrypoint 'flake.nix' or 'default.nix' found")]
    NoEntrypoint,
    #[error(transparent)]
    Build(#[from] NixBuildError),
    #[error("Writing cache failed: {0}")]
    Cache(#[from] std::io::Error),
    #[error("{0}")]
    Failed(String),
    #[error("TODO: {0}")]
    Todo(&'static str),
}

type BuildResult<T> = Result<T, BuildError>;

fn reset_bin_dir(bin_dir: &Path) -> BuildResult<()> {
    recreate_dir(bin_dir).map_err(|_| BuildError::Todo("reset_bin_dir"))
}

fn write_bin_dir(bin_dir: &Path, config: &Config) -> BuildResult<()> {
    let cache_dir = {
        let dir = config.state_dir();

        dir.join("cache")
    };

    let bins = find_bin_attrnames(config);
    let cached_sh = include_str!("cached.sh");

    for bin in bins {
        let path = bin_dir.join(&bin.0);

        let buf = {
            let mut buf = String::new();

            let this_mrx_bin =
                std::env::current_exe().map_err(|_| BuildError::Todo("current_exe"))?;

            let env_vars = [
                ("THIS_MRX_BIN", this_mrx_bin.to_string_lossy()),
                ("DERIVATION", bin.to_string().into()),
                ("CACHE_DIR", cache_dir.to_string_lossy()),
            ];

            for (k, v) in env_vars.into_iter() {
                writeln!(&mut buf, "export {k}={v}")
                    .map_err(|_| BuildError::Todo("write_bin_dir"))?;
            }

            write!(&mut buf, "\n{cached_sh}").map_err(|_| BuildError::Todo("write_cached_sh"))?;

            buf
        };

        std::fs::write(&path, buf.as_bytes())?;

        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

pub fn build(config: Config, options: Options) -> BuildResult<Vec<String>> {
    let installables = config.get_installables();

    let build_command = config
        .get_entrypoint()
        .map(|entrypoint| NixBuildCommand::new(entrypoint, installables))
        .ok_or(BuildError::NoEntrypoint)?;

    let mut paths = build_command
        .execute()?
        .into_iter()
        .map(|NixBuildOutput { bin, out }| {
            bin.or(out.map(|out| format!("{out}/bin")))
                .expect("bin or out must be Some")
        })
        .collect::<Vec<_>>();

    if options.cache {
        let bin_dir = {
            let dir = config.state_dir();

            dir.join("bin")
        };

        reset_bin_dir(&bin_dir)?;
        write_bin_dir(&bin_dir, &config)?;

        // If sourced by PATH_add in order,
        // any derivation in a symlinkJoin, built via '${INSTALLABLES}',
        // will be in preferential order in PATH, and shadow the cache-aside
        // implementation in [`bin_dir`].
        // This enables opting out of caching on a per-exe basis.
        paths.insert(0, bin_dir.to_string_lossy().to_string());
    }

    Ok(paths)
}
