use crate::{attr::Attrname, find_nix_path_attrset, Config};

#[must_use]
pub fn find_bin_attrnames(config: &Config) -> Vec<Attrname> {
    find_nix_path_attrset(config)
        .iter()
        .filter(|(_, attr)| attr.is_bin())
        .map(|(attrname, _)| attrname)
        .cloned()
        .collect::<Vec<_>>()
}
