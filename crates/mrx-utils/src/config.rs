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
    path: PathBuf,
    toml: ConfigToml,

    default_generated_out_path: PathBuf,
    default_installables: Vec<String>,
    default_entrypoint: Option<Entrypoint>,
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
            path,
            toml,
            default_generated_out_path: PathBuf::from("mrx.generated.nix"),
            default_installables: vec![],
            default_entrypoint,
        })
    }
}

#[derive(Debug, ThisError)]
pub enum ConfigValueError {
    #[error("ConfigValueError::MissingValue: '{0}'")]
    MissingValue(String),
    #[error("ConfigValudError::Io: '{0}'")]
    Io(#[from] std::io::Error),
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

type ConfigValueResult<T> = Result<T, ConfigValueError>;

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

    /// # Errors
    /// Errors if `ignore_file` is not present in the config, since there is no default.
    pub fn get_ignore_file(&self) -> ConfigValueResult<&PathBuf> {
        self.toml
            .ignore_file
            .as_ref()
            .ok_or(ConfigValueError::MissingValue("ignore_file".to_string()))
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

// TODO: This one
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
