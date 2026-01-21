use thiserror::Error;

use crate::{
    ast::NixAstNodesError,
    attr::AttrnameError,
    fs::AbsolutePathBufError,
};

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("No fallback entrypoint 'flake.nix' or 'default.nix' found")]
    NoEntrypoint,
    #[error("Missing node: {0}")]
    MissingNode(String),
    #[error("Invalid node: {0}")]
    InvalidNode(String),
    #[error("Ast node error: {0}")]
    AstNodeError(#[from] NixAstNodesError),
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

impl From<AbsolutePathBufError> for GraphError {
    fn from(value: AbsolutePathBufError) -> Self {
        match value {
            AbsolutePathBufError::Io(_, e) => Self::Io(e),
            AbsolutePathBufError::NotSupported(path) => {
                Self::InvalidNode(path.to_string_lossy().to_string())
            }
            AbsolutePathBufError::NotFound(path) => {
                Self::MissingNode(path.to_string_lossy().to_string())
            }
        }
    }
}
