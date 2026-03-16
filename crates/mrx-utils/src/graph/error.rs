use thiserror::Error as ThisError;

use crate::{
    ast::NixAstNodesError,
    attr::AttrnameError,
};

// TODO: This one
#[derive(Debug, ThisError)]
pub enum GraphError {
    #[error("GraphError::NoEntrypoint:: 'flake.nix' or 'default.nix' not found")]
    NoEntrypoint,
    #[error("GraphError::MissingNode: {0}")]
    MissingNode(String),
    #[error("GraphError::InvalidNode: {0}")]
    InvalidNode(String),
    #[error("GraphError::AstNodeError: {0}")]
    AstNodeError(#[from] NixAstNodesError),
    #[error("GraphError::Io: {0}")]
    Io(std::io::Error),
}

impl From<AttrnameError> for GraphError {
    fn from(value: AttrnameError) -> Self {
        match value {
            AttrnameError::Path(buf) => Self::InvalidNode(buf.to_string_lossy().to_string()),
            AttrnameError::Name(name) => Self::InvalidNode(name),
        }
    }
}
