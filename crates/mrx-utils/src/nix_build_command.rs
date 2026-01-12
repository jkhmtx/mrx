use std::io::Write as __;

use serde::de::Error;
use thiserror::Error;

use crate::config::Entrypoint;

#[derive(Debug)]
pub struct NixBuildCommand<'a> {
    entrypoint: Entrypoint,
    derivations: &'a [String],
}

#[derive(Debug, Error)]
pub enum NixBuildError {
    #[error("Failed to run nix build command: 'nix {command_string}'")]
    Command {
        command_string: String,
        #[source]
        io_err: std::io::Error,
    },
    #[error("Failed to deserialize: {0}")]
    Deserialization(#[from] serde_json::Error),
    #[error("nix build command failed: {0}")]
    Failed(String),
}

#[derive(Debug)]
pub struct NixBuildOutput {
    pub bin: Option<String>,
    pub out: Option<String>,
}

impl TryFrom<&serde_json::Value> for NixBuildOutput {
    type Error = serde_json::Error;

    fn try_from(item: &serde_json::Value) -> Result<Self, Self::Error> {
        item.get("outputs")
            .and_then(|v| v.as_object())
            .ok_or(serde_json::error::Error::custom("Expected JSON object"))
            .and_then(|value| {
                let out = value
                    .get("out")
                    .and_then(|v| v.as_str().map(ToOwned::to_owned));
                let bin = value
                    .get("bin")
                    .and_then(|v| v.as_str().map(ToOwned::to_owned));

                match (out, bin) {
                    (None, None) => Err(serde_json::error::Error::custom(
                        "Expected 'out' or 'bin' field",
                    )),
                    (out, bin) => Ok(NixBuildOutput { bin, out }),
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

impl NixBuildCommand<'_> {
    /// # Errors
    /// TODO
    /// # Panics
    /// TODO
    pub fn execute(self) -> Result<Vec<NixBuildOutput>, NixBuildError> {
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
            let mut stdin = build_cmd
                .stdin
                .take()
                .expect("stdin handle was not properly provided");

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
        }

        let output = build_cmd
            .wait_with_output()
            .map_err(|e| NixBuildError::Command {
                command_string: args.join(" "),
                io_err: e,
            })?;

        if !output.status.success() {
            let err_out = String::from_utf8_lossy(&output.stderr);

            return Err(NixBuildError::Failed(err_out.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        serde_json::from_str::<serde_json::Value>(&stdout)?
            .as_array()
            .ok_or(serde_json::error::Error::custom("Expected JSON array"))
            .and_then(|array| {
                array
                    .iter()
                    .map(NixBuildOutput::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })
            .map_err(NixBuildError::Deserialization)
    }
}
