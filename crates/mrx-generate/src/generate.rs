use std::fmt::Write as _;

use exn::{
    ResultExt as _,
    bail,
};
use mrx_utils::fs::{
    mk_dir,
    write_with_rollback,
};
use mrx_utils::{
    Attrname,
    Config,
    NixAst,
    NixAstNodes,
    PathAttrset,
    find_nix_path_attrset,
};
use thiserror::Error as ThisError;
use tracing::info;

use crate::Options;

#[derive(Debug, ThisError)]
pub(crate) enum GenerateError {
    #[error("GenerateError::GettingPathAttrset")]
    GettingPathAttrset,
    #[error("GenerateError::InvalidDestination: '{0}'")]
    InvalidDestination(&'static str),
    #[error("GenerateError::WriteBarrelFile")]
    WriteBarrelFile,
    #[error("GenerateError::WriteBin: could not write bin for '{0}'")]
    WriteBin(Attrname),
}

type GenerateResult<T> = Result<T, exn::Exn<GenerateError>>;

fn write_barrel_file(config: &Config, attrset: &PathAttrset) -> GenerateResult<()> {
    let out_path = config.get_generated_out_path();
    let destination = config.dir().join(out_path);
    let generated_dir = destination.parent();

    if let Some(dir) = generated_dir {
        mk_dir(dir).or_raise(|| {
            GenerateError::InvalidDestination("could not generate destination directory")
        })?;
    } else {
        bail!(GenerateError::InvalidDestination("'/' has no parent"))
    }

    let num_components = destination.components().count();

    let buf = {
        let prefix = (0..(num_components.saturating_sub(2)))
            .map(|_| "../")
            .collect::<Vec<_>>()
            .join("");

        let mut buf = String::new();

        writeln!(&mut buf, "{{").or_raise(|| GenerateError::WriteBarrelFile)?;

        let mut attrnames = attrset.keys().cloned().collect::<Vec<_>>();
        attrnames.sort();

        // TODO: Adapt this to a per-workspace presentation
        let (root_attrnames, _non_root_attrnames): (Vec<_>, Vec<_>) = attrnames
            .into_iter()
            //
            .partition(|_| true);

        for name in &root_attrnames {
            let path = attrset.get(name).unwrap().as_path().to_str().unwrap();

            let name = name.replacen("_.", "", 1);
            writeln!(&mut buf, "  {name} = {prefix}{path};")
                .or_raise(|| GenerateError::WriteBarrelFile)?;
        }

        writeln!(&mut buf, "}}").or_raise(|| GenerateError::WriteBarrelFile)?;

        buf
    };

    write_with_rollback(buf.as_bytes(), &destination).or_raise(|| GenerateError::WriteBarrelFile)
}

fn write_name_files(attrset: &PathAttrset) -> GenerateResult<()> {
    let name_dir_pairs = attrset.iter().filter_map(|(name, path_attr)| {
        NixAstNodes::new(path_attr)
            .ok()?
            .iter()
            .find(|node| matches!(node, NixAst::ImportOwnNameModuleExpression))
            .map(|_| (name, path_attr.as_path().parent().unwrap().join("_/name")))
    });

    for (attr_name, name_dir) in name_dir_pairs {
        info!("{}: {}", attr_name, name_dir.display());
        mk_dir(&name_dir).or_raise(|| GenerateError::WriteBin(attr_name.clone()))?;

        let name = {
            let mut name = String::new();

            writeln!(&mut name, "# GENERATED CODE")
                .and_then(|()| writeln!(&mut name, "\"{attr_name}\""))
                .map(|()| name)
        }
        .or_raise(|| GenerateError::WriteBin(attr_name.clone()))?;

        let path = name_dir.join("default.nix");
        if let Ok(buf) = std::fs::read(&path)
            && buf.as_slice() == name.as_bytes()
        {
            continue;
        }

        std::fs::write(&path, name.as_bytes())
            .or_raise(|| GenerateError::WriteBin(attr_name.clone()))?;
    }

    Ok(())
}

/// # Errors
/// See [`GenerateError`].
pub(crate) fn generate(config: &Config, _options: &Options) -> GenerateResult<()> {
    let attrset = find_nix_path_attrset(config).or_raise(|| GenerateError::GettingPathAttrset)?;

    write_barrel_file(config, &attrset)?;
    write_name_files(&attrset)?;

    Ok(())
}
