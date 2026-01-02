use std::{
    collections::HashMap,
    fmt::Display,
    ops::{
        Deref,
        DerefMut,
    },
    path::{
        Path,
        PathBuf,
    },
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AttrnameError {
    #[error("Invalid attrname: {0}")]
    Name(String),
    #[error("Invalid path: {0}")]
    Path(PathBuf),
}

type AttrnameDeref = String;

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct Attrname(pub AttrnameDeref);

impl Display for Attrname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for Attrname {
    type Target = AttrnameDeref;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Attrname {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<&Path> for Attrname {
    type Error = AttrnameError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if !value.starts_with("./") {
            return Err(AttrnameError::Path(value.to_path_buf()));
        }

        let mut name = value
            .components()
            .skip(1)
            .take_while(|c| c.as_os_str() != "main.nix")
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join(".");

        for (from, to) in [
            ("scripts.bin.", ""),
            ("scripts.lib.", "lib."),
            ("scripts.util.", "util."),
        ]
        .map(|(a, b)| (a.to_string(), b.to_string()))
        {
            name = name.replace(&from, &to);
        }

        Ok(Self("_.".to_string() + &name))
    }
}

impl TryFrom<&str> for Attrname {
    type Error = AttrnameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl TryFrom<String> for Attrname {
    type Error = AttrnameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.starts_with("_.") {
            Ok(Self(value))
        } else {
            Err(AttrnameError::Name(value))
        }
    }
}

impl Attrname {
    pub fn is_internal(&self) -> bool {
        self.starts_with("_.mrx")
    }
}

#[derive(Debug)]
enum AttrKind {
    Bin,
    Lib,
    Other,
    Pkg,
}

#[derive(Debug)]
pub struct PathAttr {
    path: PathBuf,
    kind: AttrKind,
}

impl PathAttr {
    pub fn as_path(&self) -> &Path {
        self.path.as_path()
    }
}

fn has_path_parts(path: &str, parts: &str) -> bool {
    path.contains(("/".to_string() + parts).as_str()) || path.starts_with(parts)
}

impl PathAttr {
    fn new(path: &Path) -> Self {
        let path_as_str = path.to_string_lossy();
        let kind = {
            if has_path_parts(&path_as_str, "scripts/bin/") {
                AttrKind::Bin
            } else if has_path_parts(&path_as_str, "scripts/lib/") {
                AttrKind::Lib
            } else if has_path_parts(&path_as_str, "pkg/") {
                AttrKind::Pkg
            } else {
                AttrKind::Other
            }
        };

        Self {
            path: path.to_path_buf(),
            kind,
        }
    }

    pub fn is_bin(&self) -> bool {
        matches!(self.kind, AttrKind::Bin)
    }
}

#[derive(Debug, Error)]
pub enum PathAttrsetError {
    #[error("{0}")]
    Attrname(#[from] AttrnameError),
}

type PathAttrsetDeref = HashMap<Attrname, PathAttr>;

#[derive(Debug, Default)]
pub struct PathAttrset(PathAttrsetDeref);

impl PathAttrset {
    /// # Errors
    /// An error is returned if any of the paths are not relative paths beginning with "./"
    pub fn new(paths: impl IntoIterator<Item = PathBuf>) -> Result<Self, PathAttrsetError> {
        let mut attrset = Self(PathAttrsetDeref::new());

        for path in paths {
            attrset.add(&path)?;
        }

        Ok(attrset)
    }

    fn add(&mut self, path: &Path) -> Result<(), PathAttrsetError> {
        self.insert(Attrname::try_from(path)?, PathAttr::new(path));

        Ok(())
    }
}

impl Deref for PathAttrset {
    type Target = PathAttrsetDeref;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PathAttrset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
