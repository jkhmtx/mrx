use std::path::PathBuf;

use thiserror::Error;

use crate::fs::AbsoluteFilePathBufError;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("No fallback entrypoint 'flake.nix' or 'default.nix' found")]
    NoEntrypoint,
    #[error("Illegal node")]
    MissingNode(PathBuf),
    #[error("Invalid node")]
    InvalidNode(PathBuf),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<AbsoluteFilePathBufError> for GraphError {
    fn from(value: AbsoluteFilePathBufError) -> Self {
        match value {
            AbsoluteFilePathBufError::Io(_, e) => Self::Io(e),
            AbsoluteFilePathBufError::NotAFile(path) => Self::InvalidNode(path),
            AbsoluteFilePathBufError::NotFound(path) => Self::MissingNode(path),
        }
    }
}
