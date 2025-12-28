use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use crate::Config;

type AttrnameDeref = String;

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct Attrname(pub AttrnameDeref);

impl Attrname {
    fn new(_config: &Config, path: &Path) -> Self {
        let mut name = path
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

        Self(name)
    }
}

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
    pub fn as_path(&self) -> &PathBuf {
        &self.path
    }
}

fn has_path_parts(path: &str, parts: &str) -> bool {
    path.contains(("/".to_string() + parts).as_str()) || path.starts_with(parts)
}

impl PathAttr {
    fn new(path: &Path) -> Self {
        let path_as_str = path.to_string_lossy();
        Self {
            path: path.to_path_buf(),
            kind: {
                if has_path_parts(&path_as_str, "scripts/bin/") {
                    AttrKind::Bin
                } else if has_path_parts(&path_as_str, "scripts/lib/") {
                    AttrKind::Lib
                } else if has_path_parts(&path_as_str, "pkg/") {
                    AttrKind::Pkg
                } else {
                    AttrKind::Other
                }
            },
        }
    }

    pub fn is_bin(&self) -> bool {
        matches!(self.kind, AttrKind::Bin)
    }
}

type PathAttrsetDeref = HashMap<Attrname, PathAttr>;

#[derive(Debug)]
pub struct PathAttrset<'a>(PathAttrsetDeref, &'a Config);

impl<'a> PathAttrset<'a> {
    #[must_use]
    pub fn new(config: &'a Config, paths: &[PathBuf]) -> PathAttrset<'a> {
        let mut attrset = Self(PathAttrsetDeref::new(), config);

        for path in paths {
            attrset.add(path);
        }

        attrset
    }

    fn add(&mut self, path: &Path) {
        let config = self.1.clone();

        self.insert(Attrname::new(&config, path), PathAttr::new(path));
    }
}

impl Deref for PathAttrset<'_> {
    type Target = PathAttrsetDeref;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PathAttrset<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
