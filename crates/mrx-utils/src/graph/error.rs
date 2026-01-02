use thiserror::Error;

use crate::{
    attr::AttrnameError,
    fs::AbsoluteFilePathBufError,
};

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("No fallback entrypoint 'flake.nix' or 'default.nix' found")]
    NoEntrypoint,
    #[error("Missing node: {0}")]
    MissingNode(String),
    #[error("Invalid node: {0}")]
    InvalidNode(String),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<AttrnameError> for GraphError {
    fn from(value: AttrnameError) -> Self {
        match value {
            AttrnameError::Path(buf) => Self::InvalidNode(buf.to_string_lossy().to_string()),
            AttrnameError::Name(name) => Self::InvalidNode(name),
        }
    }
}

impl From<AbsoluteFilePathBufError> for GraphError {
    fn from(value: AbsoluteFilePathBufError) -> Self {
        match value {
            AbsoluteFilePathBufError::Io(_, e) => Self::Io(e),
            AbsoluteFilePathBufError::NotAFile(path) => {
                Self::InvalidNode(path.to_string_lossy().to_string())
            }
            AbsoluteFilePathBufError::NotFound(path) => {
                Self::MissingNode(path.to_string_lossy().to_string())
            }
        }
    }
}
