use snafu::ensure;

use crate::{
    error::{GenerateError, RequiredItemMissingSnafu},
    manifest::{Instruction, Manifest},
};

pub fn parse_representative_manifest(
    version: &str,
    l_statements: &str,
    logic_op: &str,
    condition_op: &str,
) -> Result<Manifest, GenerateError> {
    ensure!(
        l_statements.contains("@RegisterStatement(\"set\")"),
        RequiredItemMissingSnafu {
            item: "set statement registration"
        }
    );
    ensure!(
        contains_enum_variant(logic_op, "add"),
        RequiredItemMissingSnafu {
            item: "LogicOp.add"
        }
    );
    ensure!(
        contains_enum_variant(logic_op, "not"),
        RequiredItemMissingSnafu {
            item: "LogicOp.not"
        }
    );
    ensure!(
        contains_enum_variant(condition_op, "equal"),
        RequiredItemMissingSnafu {
            item: "ConditionOp.equal"
        }
    );
    ensure!(
        contains_enum_variant(condition_op, "always"),
        RequiredItemMissingSnafu {
            item: "ConditionOp.always"
        }
    );

    Ok(Manifest {
        version: version.to_string(),
        instructions: vec![
            Instruction {
                family: "set".to_string(),
                variant: "set".to_string(),
                rust_name: "set".to_string(),
                emit: strings(["set", "$target", "$source"]),
                receiver: "target".to_string(),
                inputs: strings(["source"]),
                outputs: Vec::new(),
            },
            Instruction {
                family: "op".to_string(),
                variant: "add".to_string(),
                rust_name: "op_add".to_string(),
                emit: strings(["op", "add", "$out", "$lhs", "$rhs"]),
                receiver: String::new(),
                inputs: strings(["lhs", "rhs"]),
                outputs: strings(["out"]),
            },
            Instruction {
                family: "op".to_string(),
                variant: "not".to_string(),
                rust_name: "op_not".to_string(),
                emit: strings(["op", "not", "$out", "$input", "0"]),
                receiver: String::new(),
                inputs: strings(["input"]),
                outputs: strings(["out"]),
            },
            Instruction {
                family: "jump".to_string(),
                variant: "equal".to_string(),
                rust_name: "jump_equal".to_string(),
                emit: strings(["jump", "$label", "equal", "$lhs", "$rhs"]),
                receiver: String::new(),
                inputs: strings(["label", "lhs", "rhs"]),
                outputs: Vec::new(),
            },
            Instruction {
                family: "jump".to_string(),
                variant: "always".to_string(),
                rust_name: "jump_always".to_string(),
                emit: strings(["jump", "$label", "always", "0", "0"]),
                receiver: String::new(),
                inputs: strings(["label"]),
                outputs: Vec::new(),
            },
        ],
    })
}

fn contains_enum_variant(source: &str, variant: &str) -> bool {
    let pattern = format!("{variant}(");
    source.contains(&pattern)
}

fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.into_iter().map(ToString::to_string).collect()
}
