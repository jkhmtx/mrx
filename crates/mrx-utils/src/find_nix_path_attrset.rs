use std::{
    path::PathBuf,
    result::Result,
};

use ignore::WalkBuilder;

use crate::{
    Config,
    ConfigValueError,
    PathAttrset,
};

fn get_config_ignore(config: &Config) -> Option<&PathBuf> {
    config
        .get_ignore_file()
        .map_err(|e| match e {
            ConfigValueError::MissingValue(_) => {
                PathBuf::new().join(config.dir()).join("mrx.ignore.lst")
            }
            ConfigValueError::Io(e) => {
                panic!("{e:?}");
            }
        })
        .ok()
        .and_then(|path| if path.exists() { Some(path) } else { None })
}

/// # Panics
/// Panics if [`WalkBuilder`] somehow gives non-relative paths to [`PathAttrset::new`]
#[must_use]
pub fn find_nix_path_attrset(config: &Config) -> PathAttrset {
    let mut builder = WalkBuilder::new(config.dir());
    builder.filter_entry(|entry| {
        entry.path().is_dir() || entry.file_name().to_string_lossy() == "main.nix"
    });

    if let Some(ignore) = get_config_ignore(config) {
        builder.add_custom_ignore_filename(ignore);
    }

    let paths = builder
        .build()
        .filter_map(Result::ok)
        .filter(|dir_entry| dir_entry.file_type().is_some_and(|ft| !ft.is_dir()))
        .map(|dir_entry| dir_entry.path().to_owned());

    PathAttrset::new(paths).expect("Invalid paths were given by 'builder'")
}
