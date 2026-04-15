use exn::ResultExt;
use mrx_utils::{
    Attrname,
    Config,
    find_bin_attrnames,
};
use thiserror::Error as ThisError;

use crate::Options;

#[derive(Debug, ThisError)]
pub(crate) enum HookError {
    #[error("HookError::FindBinAttrnames")]
    FindBinAttrnames,
}

pub(crate) fn hook(
    config: &Config,
    _options: &Options,
) -> Result<Vec<Attrname>, exn::Exn<HookError>> {
    let mut bins = find_bin_attrnames(config).or_raise(|| HookError::FindBinAttrnames)?;

    bins.sort();
    Ok(bins)
}
