use std::io::Write as __;

use exn::ResultExt as _;
use exn::bail;
use serde::de::Error;
use thiserror::Error as ThisError;

use crate::{
    config::Entrypoint,
    nix_store_path::NixStorePath,
};

#[derive(Debug)]
pub struct NixBuildCommand<'a> {
    entrypoint: Entrypoint,
    derivations: &'a [String],
}

#[derive(Debug, ThisError)]
pub enum NixBuildError {
    /// This error is yieled when the command fails to start due to an IO or permission problem.
    #[error("NixBuildError::Command: 'nix {command_string}'")]
    Command {
        command_string: String,
        #[source]
        io_err: std::io::Error,
    },
    /// This error is yieled when there is an issue with deserializing the JSON stdout given by the underlying nix command.
    #[error("NixBuildError::Deserialization")]
    Deserialization,
    /// This error is yieled when there is an error in the execution of the nix command.
    #[error("NixBuildError::Failed: {0}")]
    Failed(String),
}

#[derive(Debug)]
pub struct NixBuildOutput {
    pub bin: Option<NixStorePath>,
    pub out: Option<NixStorePath>,
}

fn get_nix_store_path(
    value: &serde_json::map::Map<String, serde_json::Value>,
    key: &'static str,
) -> Option<NixStorePath> {
    value
        .get(key)
        .and_then(|v| v.as_str().map(ToOwned::to_owned).map(NixStorePath::new))
}

impl TryFrom<&serde_json::Value> for NixBuildOutput {
    type Error = serde_json::Error;

    fn try_from(item: &serde_json::Value) -> Result<Self, Self::Error> {
        item.get("outputs")
            .and_then(|v| v.as_object())
            .ok_or(serde_json::error::Error::custom("Expected JSON object"))
            .and_then(|value| {
                match (
                    get_nix_store_path(value, "bin"),
                    get_nix_store_path(value, "out"),
                ) {
                    (None, None) => Err(serde_json::error::Error::custom(
                        "Expected 'out' or 'bin' field",
                    )),
                    (bin, out) => Ok(NixBuildOutput { bin, out }),
                }
            })
    }
}

impl<'a> NixBuildCommand<'a> {
    #[must_use]
    pub fn new(entrypoint: Entrypoint, derivations: &'a [String]) -> Self {
        Self {
            entrypoint,
            derivations,
        }
    }
}

type NixBuildResult<T> = Result<T, exn::Exn<NixBuildError>>;

impl NixBuildCommand<'_> {
    /// # Errors
    /// See [`NixBuildError`].
    pub fn execute(self) -> NixBuildResult<Vec<NixBuildOutput>> {
        let mut args: Vec<String> = ["build", "--no-warn-dirty", "--json", "--no-link"]
            .into_iter()
            .map(ToString::to_string)
            .collect();

        let input_string = if self.derivations.is_empty() {
            None
        } else {
            Some(self.derivations.join("\n"))
        };

        if input_string.is_some() {
            args.push("--stdin".to_string());
        }

        if let Entrypoint::File(path) = self.entrypoint {
            args.push("--file".to_string());
            args.push(path.to_string_lossy().to_string());
        }

        let mut build_cmd = std::process::Command::new("nix")
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| NixBuildError::Command {
                command_string: args.join(" "),
                io_err: e,
            })?;

        if let Some(input_string) = input_string {
            if let Some(mut stdin) = build_cmd.stdin.take() {
                let args = args.clone();
                if let Err(e) = std::thread::spawn(move || {
                    stdin
                        .write_all(input_string.as_bytes())
                        .map_err(|e| NixBuildError::Command {
                            command_string: args.join(" "),
                            io_err: e,
                        })
                })
                .join()
                {
                    std::panic::resume_unwind(e);
                }
            } else {
                unreachable!("stdin handle was not properly provided")
            }
        }

        let output = build_cmd
            .wait_with_output()
            .map_err(|e| NixBuildError::Command {
                command_string: args.join(" "),
                io_err: e,
            })?;

        if !output.status.success() {
            let err_out = String::from_utf8_lossy(&output.stderr);

            bail!(NixBuildError::Failed(err_out.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let deserialized = serde_json::from_str::<serde_json::Value>(&stdout)
            .or_raise(|| NixBuildError::Deserialization)?;

        let array = deserialized
            .as_array()
            .ok_or(serde_json::error::Error::custom("Expected JSON array"))
            .or_raise(|| NixBuildError::Deserialization)?;

        array
            .iter()
            .map(NixBuildOutput::try_from)
            .collect::<Result<Vec<_>, _>>()
            .or_raise(|| NixBuildError::Deserialization)
    }
}
