use std::fmt::Write as _;
use std::os::unix::fs::PermissionsExt as UnixPermissions;
use std::path::Path;

use mrx_utils::fs::recreate_dir;
use mrx_utils::nix_build_command::{
    NixBuildCommand,
    NixBuildError,
    NixBuildOutput,
};
use mrx_utils::nix_store_path::NixStorePath;
use mrx_utils::{
    Config,
    find_bin_attrnames,
};
use thiserror::Error;

use crate::cli::Options;

#[derive(Debug, Error)]
pub(crate) enum BuildError {
    #[error("No entrypoint 'flake.nix' or 'default.nix' found")]
    NoEntrypoint,
    #[error(transparent)]
    Build(#[from] NixBuildError),
    #[error("Writing cache failed: {0}")]
    Cache(#[from] std::io::Error),
    #[error("TODO: {0}")]
    Todo(&'static str),
}

type BuildResult<T> = Result<T, BuildError>;

fn reset_bin_dir(bin_dir: &Path) -> BuildResult<()> {
    recreate_dir(bin_dir).map_err(|_| BuildError::Todo("reset_bin_dir"))
}

fn write_bin_dir(bin_dir: &Path, config: &Config) -> BuildResult<()> {
    let bins = find_bin_attrnames(config);
    let cached_sh = include_str!("cached.sh");

    for bin in bins {
        let path = bin_dir.join(&bin.0);

        let buf = {
            let mut buf = String::new();

            let this_mrx_bin =
                std::env::current_exe().map_err(|_| BuildError::Todo("current_exe"))?;

            let env_vars = [
                ("__MRX_DERIVATION", bin.to_string().into()),
                ("__MRX_THIS_MRX_BIN", this_mrx_bin.to_string_lossy()),
            ];

            for (k, v) in env_vars {
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

/// # Errors
/// TODO
/// # Panics
/// TODO
pub(crate) fn build(config: &Config, options: &Options) -> BuildResult<Vec<String>> {
    if options.generate {
        mrx_generate::run(config, &mrx_generate::Options::default());
    }

    let installables = config.get_installables();

    let build_command = config
        .get_entrypoint()
        .map(|entrypoint| NixBuildCommand::new(entrypoint, installables))
        .ok_or(BuildError::NoEntrypoint)?;

    let mut paths = build_command
        .execute()?
        .into_iter()
        .map(|NixBuildOutput { bin, out }| {
            bin.or(out.map(|path| NixStorePath::BinDir(path.into_string() + "/bin")))
                .map(NixStorePath::into_string)
                .expect("bin or out must be Some")
        })
        .collect::<Vec<_>>();

    if !options.skip_bin {
        let bin_dir = {
            let dir = config.state_dir();

            dir.join("bin")
        };

        reset_bin_dir(&bin_dir)?;
        write_bin_dir(&bin_dir, config)?;

        // If sourced by PATH_add in order,
        // any derivation in a symlinkJoin, built via '${INSTALLABLES}',
        // will be in preferential order in PATH, and shadow the cache-aside
        // implementation in [`bin_dir`].
        // This enables opting out of caching on a per-exe basis.
        paths.insert(0, bin_dir.to_string_lossy().to_string());
    }

    // If 'skip_bin', there is no hook to show because there are no bins available.
    if options.hook && !options.skip_bin {
        mrx_hook::run(config, &mrx_hook::Options::default());
    }

    Ok(paths)
}
