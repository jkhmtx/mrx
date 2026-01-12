use thiserror as _;

mod cli;
mod hook;
mod run;
pub use cli::Options;
pub use run::run;
