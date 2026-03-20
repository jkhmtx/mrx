use exn::bail;
use thiserror::Error as ThisError;

use crate::nix_store_path::NixStorePath;

#[derive(Debug)]
pub struct NixReferencesCommand<'a> {
    store_paths: &'a [NixStorePath],
}

#[derive(Debug, ThisError)]
pub enum NixReferencesError {
    #[error("NixReferencesError::Command: 'nix-store {command_string}'")]
    Command {
        command_string: String,
        #[source]
        io_err: std::io::Error,
    },
    #[error("NixReferencesError::Failed: {0}")]
    Failed(String),
}

#[derive(Debug)]
pub struct NixReferencesOutput {
    pub store_paths: Vec<NixStorePath>,
}

impl<'a> NixReferencesCommand<'a> {
    #[must_use]
    pub fn new(store_paths: &'a [NixStorePath]) -> Self {
        Self { store_paths }
    }
}

pub type NixReferencesCommandResult<T> = Result<T, exn::Exn<NixReferencesError>>;

impl NixReferencesCommand<'_> {
    /// # Errors
    /// TODO
    pub fn execute(self) -> NixReferencesCommandResult<NixReferencesOutput> {
        let mut args: Vec<&str> = vec!["--query", "--requisites"];

        if !self.store_paths.is_empty() {
            args.extend(self.store_paths.iter().map(NixStorePath::as_str));
        }

        let store_cmd = std::process::Command::new("nix-store")
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| NixReferencesError::Command {
                command_string: args.join(" "),
                io_err: e,
            })?;

        let output = store_cmd
            .wait_with_output()
            .map_err(|e| NixReferencesError::Command {
                command_string: args.join(" "),
                io_err: e,
            })?;

        if !output.status.success() {
            let err_out = String::from_utf8_lossy(&output.stderr);

            bail!(NixReferencesError::Failed(err_out.to_string()));
        }

        let store_paths = String::from_utf8_lossy(&output.stdout)
            .split_ascii_whitespace()
            .map(ToString::to_string)
            .map(NixStorePath::new)
            .collect::<Vec<_>>();

        Ok(NixReferencesOutput { store_paths })
    }
}
