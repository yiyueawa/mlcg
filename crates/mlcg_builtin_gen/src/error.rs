use std::path::PathBuf;

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum GenerateError {
    #[snafu(display("line {line} is missing token {name}"))]
    MissingToken { line: usize, name: &'static str },

    #[snafu(display("line {line} has invalid key-value token {token}"))]
    InvalidKeyValue { line: usize, token: String },

    #[snafu(display("failed to serialize manifest"))]
    Serialize { source: toml::ser::Error },

    #[snafu(display("git command failed: {command} exited with {status}"))]
    GitCommand { command: String, status: String },

    #[snafu(display("missing Mindustry cache file {path:?}"))]
    MissingCacheFile { path: PathBuf },

    #[snafu(display("failed to read source file {path:?}"))]
    ReadSource {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("required source item not found: {item}"))]
    RequiredItemMissing { item: &'static str },

    #[snafu(display("required source item not found: {item}"))]
    RequiredSourceItemMissing { item: String },

    #[snafu(display("{message}"))]
    GeneratedApi { message: String },
}
