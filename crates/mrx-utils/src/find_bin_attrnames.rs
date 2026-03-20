use crate::{
    Config,
    attr::{
        Attrname,
        PathAttrsetResult,
    },
    find_nix_path_attrset,
};

#[must_use]
pub fn find_bin_attrnames(config: &Config) -> PathAttrsetResult<Vec<Attrname>> {
    Ok(find_nix_path_attrset(config)?
        .iter()
        .filter(|(_, attr)| attr.is_bin())
        .map(|(attrname, _)| attrname)
        .cloned()
        .collect::<Vec<_>>())
}
