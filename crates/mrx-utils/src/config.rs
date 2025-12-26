use serde::Deserialize;
use std::{fs, path::PathBuf};

const DEFAULT_CONFIG_PATH: &str = "mrx.toml";

use thiserror::Error;

use crate::fs::pathbuf_if_exists;

#[derive(Deserialize, Debug, Clone)]
struct ConfigToml {
    ignore_file: Option<PathBuf>,
    generated_out_path: Option<PathBuf>,
    installables: Option<Vec<String>>,
    entrypoint: Option<PathBuf>,
}

impl ConfigToml {
    pub fn entrypoint(&self) -> Option<Entrypoint> {
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

impl Config {
    pub fn default_init() -> Result<Self, ConfigInitError> {
        Self::try_from(PathBuf::from(DEFAULT_CONFIG_PATH))
    }
}

#[derive(Debug, Error)]
pub enum ConfigValueError {
    #[error("value `{0}` is missing")]
    MissingValue(String),
    #[error("Io")]
    Io(#[from] std::io::Error),
}

#[derive(Deserialize, Debug, Clone)]
pub enum Entrypoint {
    Flake(PathBuf),
    File(PathBuf),
}

impl Entrypoint {
    pub fn as_path(&self) -> &PathBuf {
        match self {
            Self::Flake(path) | Self::File(path) => path,
        }
    }
}

impl TryFrom<PathBuf> for Entrypoint {
    type Error = ();
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        match value.file_name().and_then(|name| name.to_str()) {
            Some("flake.nix") => Ok(Self::Flake(value)),
            Some(name) if name.ends_with(".nix") => Ok(Self::File(value)),
            _ => Err(()),
        }
    }
}

type ConfigValueResult<T> = Result<T, ConfigValueError>;

impl Config {
    pub fn dir_absolute(&self) -> PathBuf {
        fs::canonicalize(self.dir()).unwrap()
    }

    pub fn dir(&self) -> PathBuf {
        self.path
            .parent()
            .filter(|p| p.exists())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("./"))
    }

    pub fn state_dir(&self) -> PathBuf {
        self.dir().join(".mrx")
    }

    pub fn get_ignore_file(&self) -> ConfigValueResult<&PathBuf> {
        self.toml
            .ignore_file
            .as_ref()
            .ok_or(ConfigValueError::MissingValue("ignore_file".to_string()))
    }

    pub fn get_generated_out_path(&self) -> &PathBuf {
        self.toml
            .generated_out_path
            .as_ref()
            .unwrap_or(&self.default_generated_out_path)
    }

    pub fn get_installables(&self) -> &[String] {
        self.toml
            .installables
            .as_ref()
            .unwrap_or(&self.default_installables)
    }

    pub fn get_entrypoint(&self) -> Option<Entrypoint> {
        let entrypoint = self.toml.entrypoint();

        entrypoint.or_else(|| self.default_entrypoint.clone())
    }
}

#[derive(Debug, Error)]
pub enum ConfigInitError {
    #[error("file `{0}` not found")]
    NotFound(PathBuf),
    #[error("invalid toml: {0}")]
    InvalidToml(#[from] toml::de::Error),
    #[error("error reading config file")]
    ReadError(#[from] std::io::Error),
}

pub type ConfigInitResult<T> = Result<T, ConfigInitError>;

impl<S: Into<String>> TryFrom<Option<S>> for Config {
    type Error = ConfigInitError;

    fn try_from(path: Option<S>) -> Result<Self, Self::Error> {
        path.map(S::into)
            .map_or_else(Self::default_init, Self::try_from)
    }
}

impl TryFrom<String> for Config {
    type Error = ConfigInitError;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        Self::try_from(PathBuf::from(path))
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = ConfigInitError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let file = fs::read(&path).map_err(|e| {
            use std::io::ErrorKind as IoErr;
            match e.kind() {
                IoErr::NotFound => ConfigInitError::NotFound(path.clone()),
                _ => ConfigInitError::ReadError(e),
            }
        })?;

        let toml: ConfigToml = toml::from_slice(&file)?;

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

pub trait MrxCli
where
    Self: Sized,
{
    fn create_mrx_cli_args() -> ConfigInitResult<(Config, Self)>;
}
