use std::{
    io::ErrorKind,
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
};

use thiserror::Error;

use crate::fs::is_nix;

#[derive(Debug)]
pub enum NixAst {
    /// Expressions with attr selection, which starts with `_`.
    /// e.g. `_.path.to.my.derivation`
    MrxDerivation { name: String },
    /// `import _/name` or `import ./_/name`
    ImportOwnNameModuleExpression,
    /// A path literal indicating a directory ending with `.`
    /// e.g. `./.`
    NixDirectoryPath { path: String },
    /// A path literal indicating a file or directory
    SimplePath { path: String },
}

impl TryFrom<&rnix::SyntaxNode> for NixAst {
    type Error = ();

    fn try_from(value: &rnix::SyntaxNode) -> Result<Self, Self::Error> {
        use rnix::SyntaxKind as Kind;

        match value.kind() {
            Kind::NODE_APPLY
                if {
                    let text = value.text();
                    text == "import _/name" || text == "import ./_/name"
                } =>
            {
                Some(Self::ImportOwnNameModuleExpression)
            }
            Kind::NODE_PATH => {
                let text = value.text();

                // If the last component in a path is the character '.',
                // it means it refers to a directory.
                // e.g. '.', '../.'
                let is_nix_directory_path = text
                    .len()
                    .checked_sub(1.into())
                    .and_then(|idx| text.char_at(idx))
                    .is_some_and(|c| c == '.');

                if is_nix_directory_path {
                    Some(Self::NixDirectoryPath {
                        path: text.to_string(),
                    })
                } else {
                    Some(Self::SimplePath {
                        path: text.to_string(),
                    })
                }
            }
            Kind::NODE_SELECT if value.first_child().is_some_and(|child| child.text() == "_") => {
                Some(Self::MrxDerivation {
                    name: value.text().to_string(),
                })
            }
            _ => None,
        }
        .ok_or(())
    }
}

fn walk(syntax_node: &rnix::SyntaxNode, nodes: &mut Vec<NixAst>) {
    if let Ok(node) = NixAst::try_from(syntax_node) {
        nodes.push(node);
    }

    for child in syntax_node.children() {
        walk(&child, nodes);
    }
}

#[derive(Debug, Error)]
pub enum NixAstNodesError {
    #[error("Not a nix file: {0}")]
    NotNix(PathBuf),
    #[error("Missing or invalid file: {0}")]
    MissingOrInvalidFile(PathBuf),
    #[error("Io error: {0}")]
    Io(std::io::Error),
}

impl<TPath: AsRef<Path>> From<(TPath, std::io::Error)> for NixAstNodesError {
    fn from((path, err): (TPath, std::io::Error)) -> Self {
        match err.kind() {
            ErrorKind::NotFound => {
                NixAstNodesError::MissingOrInvalidFile(path.as_ref().to_path_buf())
            }
            _ => NixAstNodesError::Io(err),
        }
    }
}

impl<TPath: AsRef<Path>> From<(TPath, std::string::FromUtf8Error)> for NixAstNodesError {
    fn from((path, _): (TPath, std::string::FromUtf8Error)) -> Self {
        NixAstNodesError::MissingOrInvalidFile(path.as_ref().to_path_buf())
    }
}

type NixAstNodesDeref = Vec<NixAst>;

#[derive(Debug)]
pub struct NixAstNodes(NixAstNodesDeref);

impl Deref for NixAstNodes {
    type Target = NixAstNodesDeref;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl NixAstNodes {
    /// # Errors
    /// TODO
    pub fn new(path: impl AsRef<Path>) -> Result<Self, NixAstNodesError> {
        if !is_nix(&path) {
            return Err(NixAstNodesError::NotNix(path.as_ref().to_path_buf()));
        }

        let buf = std::fs::read(&path).map_err(|e| (path.as_ref(), e))?;
        let file = String::from_utf8(buf).map_err(|e| (path, e))?;

        let root = rnix::Root::parse(&file).syntax();
        let mut nodes = vec![];
        walk(&root, &mut nodes);

        Ok(Self(nodes))
    }
}
