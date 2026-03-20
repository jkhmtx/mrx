use std::fmt::Write as _;
use std::os::unix::fs::PermissionsExt as UnixPermissions;
use std::path::Path;

use exn::ResultExt as _;
use mrx_utils::fs::recreate_dir;
use mrx_utils::nix_build_command::{
    NixBuildCommand,
    NixBuildOutput,
};
use mrx_utils::nix_store_path::NixStorePath;
use mrx_utils::{
    Config,
    find_bin_attrnames,
};
use thiserror::Error as ThisError;

use crate::cli::Options;

#[derive(Debug, ThisError)]
pub(crate) enum BuildError {
    #[error(
        "BuildError::GettingEntrypoint: custom entrypoint, 'flake.nix' or 'default.nix' not found"
    )]
    NoEntrypoint,
    #[error("BuildError::NixBuildCommand")]
    NixBuildCommand,
    #[error("BuildError::FailedToResetBinDir")]
    FailedToResetBinDir,
    #[error("BuildError::FailedToGetExe")]
    FailedToGetExe,
    #[error("BuildError::FailedToWriteBin")]
    FailedToWriteBin,
}

type BuildResult<T> = Result<T, exn::Exn<BuildError>>;

fn write_bin_dir(bin_dir: &Path, config: &Config) -> BuildResult<()> {
    let bins = find_bin_attrnames(config);
    let cached_sh = include_str!("cached.sh");

    let this_mrx_bin = std::env::current_exe().or_raise(|| BuildError::FailedToGetExe)?;

    for bin in bins {
        let path = bin_dir.join(&bin.0);

        let buf = {
            let mut buf = String::new();

            let env_vars = [
                ("__MRX_DERIVATION", bin.to_string().into()),
                ("__MRX_THIS_MRX_BIN", this_mrx_bin.to_string_lossy()),
            ];

            for (k, v) in env_vars {
                writeln!(&mut buf, "export {k}={v}").or_raise(|| BuildError::FailedToWriteBin)?;
            }

            write!(&mut buf, "\n{cached_sh}").or_raise(|| BuildError::FailedToWriteBin)?;

            buf
        };

        write_cache_file(&path, &buf).or_raise(|| BuildError::FailedToWriteBin)?;
    }

    Ok(())
}

fn write_cache_file(path: &std::path::PathBuf, buf: &str) -> Result<(), std::io::Error> {
    std::fs::write(path, buf.as_bytes())?;

    let mut perms = std::fs::metadata(path)?.permissions();
    let readonly_mode = 0o755;
    perms.set_mode(readonly_mode);

    std::fs::set_permissions(path, perms)?;
    Ok(())
}

/// # Errors
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
        .execute()
        .map_err(|err| err.raise(BuildError::NixBuildCommand))?
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

        recreate_dir(&bin_dir).or_raise(|| BuildError::FailedToResetBinDir)?;
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
