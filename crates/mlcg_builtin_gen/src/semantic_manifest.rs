use std::collections::HashSet;

use crate::{
    manifest::{Instruction, Manifest},
    raw_statement::{RawStatement, RawStatementManifest},
};

pub fn derive_semantic_manifest(raw: &RawStatementManifest) -> Manifest {
    let mut instructions = Vec::new();
    let mut superseded = HashSet::new();

    if has_statement(raw, "set") {
        instructions.push(Instruction {
            family: "set".to_string(),
            variant: "set".to_string(),
            rust_name: "set".to_string(),
            emit: strings(["set", "$target", "$source"]),
            receiver: "target".to_string(),
            inputs: strings(["source"]),
            outputs: Vec::new(),
        });
        superseded.insert("set".to_string());
    }

    if has_statement(raw, "op") {
        instructions.extend([
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
        ]);
        superseded.insert("op".to_string());
    }

    if has_statement(raw, "jump") {
        instructions.extend([
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
        ]);
        superseded.insert("jump".to_string());
    }

    instructions.extend(
        raw.statements
            .iter()
            .filter(|statement| !superseded.contains(&statement.name))
            .map(derive_generic_instruction),
    );

    Manifest {
        version: raw.version.clone(),
        instructions,
    }
}

fn has_statement(raw: &RawStatementManifest, name: &str) -> bool {
    raw.statements
        .iter()
        .any(|statement| statement.name == name)
}

fn derive_generic_instruction(statement: &RawStatement) -> Instruction {
    let output_candidates: Vec<_> = statement
        .fields
        .iter()
        .filter(|field| is_output_field(&field.name))
        .map(|field| field.name.clone())
        .collect();
    let outputs = if output_candidates.len() == 1 {
        output_candidates
    } else {
        Vec::new()
    };
    let inputs = statement
        .fields
        .iter()
        .filter(|field| !outputs.iter().any(|output| output == &field.name))
        .map(|field| field.name.clone())
        .collect();
    let mut emit = Vec::with_capacity(statement.fields.len() + 1);
    emit.push(statement.name.clone());
    emit.extend(
        statement
            .fields
            .iter()
            .map(|field| format!("${}", field.name)),
    );

    Instruction {
        family: statement.name.clone(),
        variant: statement.name.clone(),
        rust_name: rust_name(&statement.name),
        emit,
        receiver: String::new(),
        inputs,
        outputs,
    }
}

fn is_output_field(name: &str) -> bool {
    matches!(name, "output" | "result")
        || name
            .strip_prefix("out")
            .and_then(|suffix| suffix.chars().next())
            .is_some_and(char::is_uppercase)
}

fn rust_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.into_iter().map(ToString::to_string).collect()
}
