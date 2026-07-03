use snafu::{ensure, OptionExt};

use crate::{
    error::{GenerateError, InvalidKeyValueSnafu, MissingTokenSnafu},
    manifest::{Instruction, Manifest},
};

pub fn parse_fixture_manifest(version: &str, input: &str) -> Result<String, GenerateError> {
    let mut instructions = Vec::new();
    for (index, raw_line) in input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let family = tokens.next().context(MissingTokenSnafu {
            line: line_no,
            name: "family",
        })?;
        let variant = tokens.next().context(MissingTokenSnafu {
            line: line_no,
            name: "variant",
        })?;
        let mut emit = Vec::new();
        let mut receiver = String::new();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for token in tokens {
            let (key, value) = token.split_once('=').context(InvalidKeyValueSnafu {
                line: line_no,
                token: token.to_string(),
            })?;
            match key {
                "emit" => emit = split_list(value),
                "receiver" => receiver = value.to_string(),
                "inputs" => inputs = split_list(value),
                "outputs" => outputs = split_list(value),
                "label" | "unary" => {}
                _ => {
                    return InvalidKeyValueSnafu {
                        line: line_no,
                        token: token.to_string(),
                    }
                    .fail();
                }
            }
        }
        ensure!(
            !emit.is_empty(),
            MissingTokenSnafu {
                line: line_no,
                name: "emit"
            }
        );
        instructions.push(Instruction {
            family: family.to_string(),
            variant: variant.to_string(),
            rust_name: format!("{}_{}", family, variant).replace("statement_", ""),
            emit,
            receiver,
            inputs,
            outputs,
            labels: Vec::new(),
        });
    }

    Manifest {
        version: version.to_string(),
        instructions,
    }
    .to_toml()
}

fn split_list(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(ToString::to_string).collect()
    }
}
