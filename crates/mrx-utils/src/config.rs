use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

use exn::ResultExt as _;
use serde::Deserialize;

const DEFAULT_CONFIG_PATH: &str = "mrx.toml";
const DEFAULT_IGNORE_FILE_PATH: &str = "mrx.ignore";
const DEFAULT_GENERATED_OUT_PATH: &str = "mrx.generated.toml";

use thiserror::Error as ThisError;

use crate::fs::pathbuf_if_exists;

#[derive(Deserialize, Debug, Clone)]
struct ConfigToml {
    ignore_file: Option<PathBuf>,
    generated_out_path: Option<PathBuf>,
    installables: Option<Vec<String>>,
    entrypoint: Option<PathBuf>,
}

impl ConfigToml {
    pub(crate) fn entrypoint(&self) -> Option<Entrypoint> {
        self.entrypoint
            .clone()
            .map(Entrypoint::try_from)
            .and_then(Result::ok)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    default_entrypoint: Option<Entrypoint>,
    default_generated_out_path: PathBuf,
    default_ignore_file: PathBuf,
    default_installables: Vec<String>,

    path: PathBuf,
    toml: ConfigToml,
}

pub type ConfigInitResult<T> = Result<T, exn::Exn<ConfigInitError>>;

impl Config {
    /// # Errors
    /// See [`ConfigInitError`].
    pub fn default_init() -> ConfigInitResult<Self> {
        Self::try_from_str(DEFAULT_CONFIG_PATH)
    }

    /// # Errors
    /// See [`ConfigInitError`].
    pub fn try_from_str(path: impl AsRef<str>) -> ConfigInitResult<Self> {
        let path = PathBuf::from(path.as_ref());
        let file = fs::read(&path).map_err(|e| {
            use std::io::ErrorKind as IoErr;
            match e.kind() {
                IoErr::NotFound => ConfigInitError::NotFound(path.clone()),
                _ => ConfigInitError::ReadError(e),
            }
        })?;

        let toml: ConfigToml = toml::from_slice(&file).or_raise(|| ConfigInitError::InvalidToml)?;

        let default_entrypoint = pathbuf_if_exists("./flake.nix")
            .map(Entrypoint::Flake)
            .or_else(|| pathbuf_if_exists("./default.nix").map(Entrypoint::File));

        Ok(Self {
            default_entrypoint,
            default_generated_out_path: PathBuf::from(DEFAULT_GENERATED_OUT_PATH),
            default_ignore_file: PathBuf::from(DEFAULT_IGNORE_FILE_PATH),
            default_installables: vec![],
            path,
            toml,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum Entrypoint {
    Flake(PathBuf),
    File(PathBuf),
}

impl AsRef<Path> for Entrypoint {
    fn as_ref(&self) -> &Path {
        match self {
            Self::Flake(path) | Self::File(path) => path,
        }
    }
}

impl TryFrom<PathBuf> for Entrypoint {
    type Error = ();
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let is_nix_file = value
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("nix"));
        let is_flake = value
            .file_prefix()
            .is_some_and(|name| name.eq_ignore_ascii_case("flake"));

        if is_nix_file && is_flake {
            Ok(Self::Flake(value))
        } else if is_nix_file {
            Ok(Self::File(value))
        } else {
            Err(())
        }
    }
}

impl Config {
    #[must_use]
    pub fn dir(&self) -> PathBuf {
        self.path
            .parent()
            .filter(|p| p.exists())
            .map_or_else(|| PathBuf::from("./"), Path::to_path_buf)
    }

    #[must_use]
    pub fn state_dir(&self) -> PathBuf {
        self.dir().join(".mrx")
    }

    pub fn get_ignore_file(&self) -> &PathBuf {
        self.toml
            .ignore_file
            .as_ref()
            .unwrap_or(&self.default_ignore_file)
    }

    #[must_use]
    pub fn get_generated_out_path(&self) -> &PathBuf {
        self.toml
            .generated_out_path
            .as_ref()
            .unwrap_or(&self.default_generated_out_path)
    }

    #[must_use]
    pub fn get_installables(&self) -> &[String] {
        self.toml
            .installables
            .as_ref()
            .unwrap_or(&self.default_installables)
    }

    #[must_use]
    pub fn get_entrypoint(&self) -> Option<Entrypoint> {
        let entrypoint = self.toml.entrypoint();

        entrypoint.or_else(|| self.default_entrypoint.clone())
    }
}

#[derive(Debug, ThisError)]
pub enum ConfigInitError {
    #[error("ConfigInitError::NotFound: '{0}'")]
    NotFound(PathBuf),
    #[error("ConfigInitError::InvalidToml")]
    InvalidToml,
    #[error("ConfigInitError::ReadError")]
    ReadError(#[from] std::io::Error),
}

pub trait MrxCli
where
    Self: Sized,
{
    /// # Errors
    /// See [`ConfigInitError`].
    fn create_mrx_cli_args() -> ConfigInitResult<(Config, Self)>;
}
