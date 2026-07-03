use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    pub(crate) version: String,
    pub(crate) instructions: Vec<InstructionSpec>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InstructionSpec {
    pub(crate) family: String,
    pub(crate) variant: String,
    pub(crate) rust_name: String,
    pub(crate) emit: Vec<String>,
    #[serde(default)]
    pub(crate) receiver: String,
    #[serde(default)]
    pub(crate) inputs: Vec<String>,
    #[serde(default)]
    pub(crate) outputs: Vec<String>,
}
