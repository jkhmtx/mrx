use std::path::PathBuf;

use crate::Attrname;

#[derive(Debug, Clone)]
pub struct MrxNixStorePath(pub String, pub Attrname);

#[derive(Debug, Clone)]
pub enum NixStorePath {
    BinDir(String),
    Exe(String),
    OutDir(String),
    MrxBinDir(MrxNixStorePath),
    MrxExe(MrxNixStorePath),
    MrxOutDir(MrxNixStorePath),
}

fn as_attrname(path: &str) -> Option<Attrname> {
    let split = path.split_once('-');

    // SAFETY: store paths always follow the form '/nix/store/123abc-[name]'
    let (_, name) = unsafe { split.unwrap_unchecked() };

    Attrname::try_from(name).ok()
}

impl NixStorePath {
    #[must_use]
    pub fn new(path: String) -> Self {
        debug_assert!(
            path.starts_with("/nix/store/"),
            "NixStorePath::new - path must start with '/nix/store/', got {path}"
        );

        match (path.rsplit_once("/bin"), as_attrname(&path)) {
            (Some((_, "" | "/")), Some(attrname)) => {
                Self::MrxBinDir(MrxNixStorePath(path, attrname))
            }
            (Some((_, _)), Some(attrname)) => Self::MrxExe(MrxNixStorePath(path, attrname)),
            (Some((_, "" | "/")), None) => Self::BinDir(path),
            (Some((_, _)), None) => Self::Exe(path),
            (None, Some(attrname)) => Self::MrxOutDir(MrxNixStorePath(path, attrname)),
            (None, None) => Self::OutDir(path),
        }
    }

    #[must_use]
    pub fn into_string(self) -> String {
        match self {
            Self::BinDir(v)
            | Self::OutDir(v)
            | Self::Exe(v)
            | Self::MrxBinDir(MrxNixStorePath(v, _))
            | Self::MrxExe(MrxNixStorePath(v, _))
            | Self::MrxOutDir(MrxNixStorePath(v, _)) => v,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        match &self {
            Self::BinDir(v)
            | Self::OutDir(v)
            | Self::Exe(v)
            | Self::MrxBinDir(MrxNixStorePath(v, _))
            | Self::MrxExe(MrxNixStorePath(v, _))
            | Self::MrxOutDir(MrxNixStorePath(v, _)) => v,
        }
    }

    #[must_use]
    pub fn as_attrname(&self) -> Option<&Attrname> {
        match self {
            Self::MrxBinDir(MrxNixStorePath(_, attrname)) => Some(attrname),
            _ => None,
        }
    }

    #[must_use]
    pub fn to_path(&self) -> PathBuf {
        PathBuf::from(&self.as_str())
    }

    #[must_use]
    pub fn into_mrx_exe(self) -> Option<Self> {
        match self {
            NixStorePath::BinDir(_) | NixStorePath::Exe(_) | NixStorePath::OutDir(_) => None,
            NixStorePath::MrxBinDir(MrxNixStorePath(path, attrname)) => Some(NixStorePath::MrxExe(
                MrxNixStorePath(path + attrname.as_str(), attrname.clone()),
            )),
            NixStorePath::MrxExe(_) => Some(self),
            NixStorePath::MrxOutDir(MrxNixStorePath(path, attrname)) => Some(NixStorePath::MrxExe(
                MrxNixStorePath(path + "/bin/" + attrname.as_str(), attrname.clone()),
            )),
        }
    }
}
