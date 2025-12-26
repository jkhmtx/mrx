use std::path::PathBuf;

use ignore::WalkBuilder;

use crate::{Config, ConfigValueError, PathAttrset};

fn get_config_ignore(config: &Config) -> Option<&PathBuf> {
    config
        .get_ignore_file()
        .map_err(|e| match e {
            ConfigValueError::MissingValue(_) => {
                PathBuf::new().join(config.dir()).join("mrx.ignore.lst")
            }
            ConfigValueError::Io(e) => {
                panic!("{:?}", e);
            }
        })
        .ok()
        .and_then(|path| if path.exists() { Some(path) } else { None })
}

pub fn find_nix_path_attrset(config: &Config) -> PathAttrset<'_> {
    let mut builder = WalkBuilder::new(config.dir());
    builder.filter_entry(|entry| {
        entry.path().is_dir() || entry.file_name().to_string_lossy() == "main.nix"
    });

    if let Some(ignore) = get_config_ignore(config) {
        builder.add_custom_ignore_filename(ignore);
    }

    let paths = builder
        .build()
        .filter_map(|r| {
            r.ok().and_then(|d| {
                let dir = d.path();
                if dir.is_dir() {
                    None
                } else {
                    Some(dir.to_owned())
                }
            })
        })
        .collect::<Vec<_>>();

    PathAttrset::new(config, paths.into_iter().as_slice())
}
