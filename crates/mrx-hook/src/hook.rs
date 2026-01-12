use mrx_utils::{
    Attrname,
    Config,
    find_bin_attrnames,
};

use crate::Options;

pub(crate) fn hook(config: &Config, _options: &Options) -> Vec<Attrname> {
    let mut bins = find_bin_attrnames(config);

    bins.sort();
    bins
}
