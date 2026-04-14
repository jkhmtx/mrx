use std::{
    error::Error,
    fmt::Display,
    fs::{
        self,
        Metadata,
    },
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
};

use exn::OptionExt as _;
use thiserror::Error as ThisError;

use crate::attr::PathAttr;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum AbsolutePathBuf {
    File(PathBuf),
    Directory(PathBuf),
}

impl Deref for AbsolutePathBuf {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        match &self {
            Self::File(p) | Self::Directory(p) => p,
        }
    }
}

#[derive(Debug, ThisError)]
pub enum AbsolutePathBufError {
    #[error("AbsolutePathBufError::NotFound")]
    NotFound,
    #[error("AbsolutePathBufError::Canonicalizing")]
    Canonicalizing,
    #[error("AbsolutePathBufError::GettingMetadata")]
    GettingMetadata,
    #[error("AbsolutePathBufError::Io: '{0}'")]
    Io(std::io::Error),
}

pub type AbsolutePathBufResult<T> = Result<T, MyExn<AbsolutePathBufError>>;

#[derive(Debug)]
pub struct MyExn<T: Error + Send + Sync + Display + 'static>(exn::Exn<T>);

impl<T: Error + Send + Sync + Display + 'static> Error for MyExn<T> {}

impl<T: Error + Send + Sync + Display + 'static> Display for MyExn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Error + Send + Sync + Display + 'static> From<T> for MyExn<T> {
    fn from(value: T) -> Self {
        Self(exn::Exn::from(value))
    }
}

impl<T: Error + Send + Sync + Display + 'static> From<exn::Exn<T>> for MyExn<T> {
    fn from(value: exn::Exn<T>) -> Self {
        Self(value)
    }
}

impl<T: Error + Send + Sync + 'static> Deref for MyExn<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<std::io::Error> for MyExn<AbsolutePathBufError> {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => MyExn(exn::Exn::from(AbsolutePathBufError::NotFound)),
            _ => MyExn(exn::Exn::from(AbsolutePathBufError::Io(value))),
        }
    }
}

fn canonicalize(path: &Path) -> AbsolutePathBufResult<PathBuf> {
    Ok(fs::canonicalize(path)?)
}

fn metadata(path: &Path) -> AbsolutePathBufResult<Metadata> {
    Ok(path.metadata()?)
}

impl TryFrom<&Path> for AbsolutePathBuf {
    type Error = MyExn<AbsolutePathBufError>;

    fn try_from(path: &Path) -> AbsolutePathBufResult<Self> {
        let default_nix = path.join("default.nix");
        let path = if default_nix.is_file() {
            &default_nix
        } else {
            path
        };

        let metadata = metadata(path)?;

        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            canonicalize(path)?
        };

        Ok(Self::new(path, &metadata))
    }
}

impl TryFrom<&PathAttr> for AbsolutePathBuf {
    type Error = MyExn<AbsolutePathBufError>;

    fn try_from(value: &PathAttr) -> AbsolutePathBufResult<Self> {
        AbsolutePathBuf::try_from(value.as_ref())
    }
}

impl AsRef<Path> for AbsolutePathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AbsolutePathBuf {
    /// Creates a new [`AbsolutePathBuf`].
    #[allow(clippy::unnecessary_debug_formatting)]
    #[must_use]
    pub fn new(path: PathBuf, metadata: &Metadata) -> Self {
        debug_assert!(
            path.has_root(),
            "AbsolutePathBuf::new - must be an absolute path: {path:?}"
        );

        if metadata.is_file() {
            return Self::File(path);
        }

        debug_assert!(
            metadata.is_dir(),
            "AbsolutePathBuf::new - path must be a file or directory, on disk: {path:?}"
        );

        Self::Directory(path)
    }

    /// # Errors
    /// Returns an error if [`parent`] is not a directory, or is not a parent of [`self`]
    pub fn as_relative_to_parent(&self, parent: &Path) -> RelativeToParentResult<PathBuf> {
        if parent.is_file() {
            return Err(MyExn::from(RelativeToParentError::InvalidParent(format!(
                "path {} is not a directory",
                parent.to_string_lossy()
            ))));
        }

        if !parent.exists() {
            return Err(MyExn::from(RelativeToParentError::InvalidParent(format!(
                "path {} does not exist",
                parent.to_string_lossy()
            ))));
        }

        let relative_to = if parent.is_absolute() {
            parent.to_path_buf()
        } else {
            match canonicalize(parent) {
                Ok(path) => Ok(path),
                Err(e) if matches!(*e, AbsolutePathBufError::NotFound) => unreachable!(),
                Err(e) => Err(RelativeToParentError::AbsolutePathBufError(e)),
            }?
        };

        let relative: Option<PathBuf> = {
            let mut path_components = self.components();
            let mut parent_components = relative_to.components();
            let suffix;
            loop {
                let mut iter_next = path_components.clone();

                let path_next = iter_next.next();
                let parent_next = parent_components.next();

                match (path_next, parent_next) {
                    (Some(ref x), Some(ref y)) if x == y => (),
                    // The directory iterator is exhausted,
                    // and we did not encounter break, which means:
                    // path_components contains the remainder after the common path
                    (Some(_), None) => {
                        suffix = Some(path_components);
                        break;
                    }
                    _ => {
                        suffix = None;
                        break;
                    }
                }
                path_components = iter_next;
            }

            suffix.map(|components| {
                let mut path = PathBuf::from("./");
                path.extend(components);

                path
            })
        };

        Ok(relative.ok_or_raise(|| {
            RelativeToParentError::InvalidParent(format!(
                "'{}' is not a parent of '{}' ",
                parent.display(),
                self.display()
            ))
        })?)
    }
}

impl Display for AbsolutePathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_str = self
            .as_path()
            .to_str()
            .ok_or_else(std::fmt::Error::default)?;

        f.write_str(display_str)
    }
}

#[derive(Debug, ThisError)]
pub enum RelativeToParentError {
    #[error("RelativeToParentError::InvalidParent: '{0}'")]
    InvalidParent(String),
    #[error("RelativeToParentError::AbsolutePathBufError: '{0}'")]
    AbsolutePathBufError(MyExn<AbsolutePathBufError>),
}

pub type RelativeToParentResult<T> = Result<T, MyExn<RelativeToParentError>>;
