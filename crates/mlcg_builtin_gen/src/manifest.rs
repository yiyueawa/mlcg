use serde::{Deserialize, Serialize};
use snafu::ResultExt;

use crate::{error::GenerateError, error::SerializeSnafu};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub version: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Instruction {
    pub family: String,
    pub variant: String,
    pub rust_name: String,
    pub emit: Vec<String>,
    pub receiver: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

impl Manifest {
    pub fn to_toml(&self) -> Result<String, GenerateError> {
        toml::to_string_pretty(self).context(SerializeSnafu)
    }
}
