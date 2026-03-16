use std::fmt::Write as _;

use exn::ResultExt as _;
use mrx_utils::fs::{
    WriteWithFallbackError,
    mk_dir,
    write_with_fallback,
};
use mrx_utils::{
    Config,
    NixAst,
    NixAstNodes,
    PathAttrset,
    find_nix_path_attrset,
};
use thiserror::Error as ThisError;

use crate::Options;

#[derive(Debug, ThisError)]
pub(crate) enum GenerateError {
    #[error("invalid destination `{0}`")]
    InvalidDestination(String),
    #[error("GenerateError::Failed: {0}")]
    Failed(String),
    #[error("Could not create file")]
    IoError(#[from] std::io::Error),
    #[error("Error constructing file string")]
    FmtError(#[from] std::fmt::Error),
}

type GenerateResult<T> = Result<T, exn::Exn<GenerateError>>;

fn write_barrel_file(config: &Config, attrset: &PathAttrset) -> GenerateResult<()> {
    let out_path = config.get_generated_out_path();
    let destination = config.dir().join(out_path);
    let generated_dir = destination.parent();

    if let Some(dir) = generated_dir {
        mk_dir(dir).or_raise(|| {
            GenerateError::Failed("could not generate destination directory".to_string())
        })?;
    } else {
        todo!("This case is reachable when config dir is the '/' directory.");
    }

    let num_components = destination.components().count();

    let buf = {
        let prefix = (0..(num_components.saturating_sub(2)))
            .map(|_| "../")
            .collect::<Vec<_>>()
            .join("");

        let mut buf = String::new();

        writeln!(&mut buf, "{{")
            .or_raise(|| GenerateError::Failed("could not write barrel file".to_string()))?;

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
                .or_raise(|| GenerateError::Failed("could not write barrel file".to_string()))?;
        }

        writeln!(&mut buf, "}}")
            .or_raise(|| GenerateError::Failed("could not write barrel file".to_string()))?;

        buf
    };

    write_with_fallback(buf.as_bytes(), &destination).map_err(|e| match e {
        WriteWithFallbackError::InvalidDest(e) => {
            exn::Exn::from(GenerateError::InvalidDestination(e.to_string()))
        }
        WriteWithFallbackError::Failed(e) | WriteWithFallbackError::RolledBack(e) => {
            exn::Exn::from(GenerateError::IoError(e))
        }
    })
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
        mk_dir(&name_dir).or_raise(|| {
            GenerateError::Failed(format!("failed to write dir for attr: '{attr_name}'"))
        })?;

        let name = {
            let mut name = String::new();

            writeln!(&mut name, "# GENERATED CODE")
                .and_then(|()| writeln!(&mut name, "\"{attr_name}\""))
                .map(|()| name)
        }
        .or_raise(|| GenerateError::Failed(format!("could not write bin for {attr_name}")))?;

        let path = name_dir.join("default.nix");
        if let Ok(buf) = std::fs::read(&path)
            && buf.as_slice() == name.as_bytes()
        {
            continue;
        }

        std::fs::write(&path, name.as_bytes())
            .or_raise(|| GenerateError::Failed(format!("could not write bin for {attr_name}")))?;
    }

    Ok(())
}

/// # Errors
/// TODO
/// # Panics
/// TODO
pub(crate) fn generate(config: &Config, _options: &Options) -> GenerateResult<()> {
    let attrset = find_nix_path_attrset(config);

    write_barrel_file(config, &attrset)?;
    write_name_files(&attrset)?;

    Ok(())
}
