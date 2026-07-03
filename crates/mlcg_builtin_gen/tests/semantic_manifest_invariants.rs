use std::fs;

use mlcg_builtin_gen::{
    generated_api::validate_generated_rust_api_symbols,
    manifest::{Instruction, Manifest},
};

#[test]
fn v158_1_semantic_manifest_classifies_every_emit_placeholder() {
    let manifest = read_v158_1_manifest();

    for instruction in manifest.instructions {
        let placeholders = emit_placeholders(&instruction.emit);
        let mut roles = Vec::new();
        if !instruction.receiver.is_empty() {
            roles.push(instruction.receiver.as_str());
        }
        roles.extend(instruction.inputs.iter().map(String::as_str));
        roles.extend(instruction.outputs.iter().map(String::as_str));
        roles.extend(instruction.labels.iter().map(String::as_str));

        for placeholder in placeholders {
            assert!(
                roles.iter().any(|role| role == &placeholder),
                "{} leaves emit placeholder ${placeholder} unclassified",
                instruction.rust_name
            );
        }

        for role in roles {
            assert!(
                instruction
                    .emit
                    .iter()
                    .any(|token| token == &format!("${role}")),
                "{} classifies non-emitted parameter {role}",
                instruction.rust_name
            );
        }
    }
}

#[test]
fn v158_1_semantic_manifest_generates_unique_rust_api_symbols() {
    let manifest = read_v158_1_manifest();

    validate_generated_rust_api_symbols(&manifest).expect("v158.1 API symbols are unique");
}

#[test]
fn generated_rust_api_symbol_validation_rejects_into_method_collision() {
    let manifest = Manifest {
        version: "fixture".to_string(),
        instructions: vec![
            Instruction {
                family: "fixture".to_string(),
                variant: "foo".to_string(),
                rust_name: "foo".to_string(),
                emit: strings(["foo", "$out"]),
                receiver: String::new(),
                inputs: Vec::new(),
                outputs: strings(["out"]),
                labels: Vec::new(),
            },
            Instruction {
                family: "fixture".to_string(),
                variant: "foo_into".to_string(),
                rust_name: "foo_into".to_string(),
                emit: strings(["foo_into"]),
                receiver: String::new(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                labels: Vec::new(),
            },
        ],
    };

    let error = validate_generated_rust_api_symbols(&manifest)
        .expect_err("processor method collision is rejected");

    assert_eq!(
        error,
        "generated processor method `foo_into` for instruction `foo_into` collides with instruction `foo`"
    );
}

fn read_v158_1_manifest() -> Manifest {
    let manifest_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../mlcg_builtin/manifests/v158_1.toml"
    );
    toml::from_str(&fs::read_to_string(manifest_path).expect("read v158.1 manifest"))
        .expect("parse v158.1 manifest")
}

fn emit_placeholders(emit: &[String]) -> Vec<&str> {
    let mut placeholders = Vec::new();
    for token in emit {
        if let Some(placeholder) = token.strip_prefix('$') {
            if !placeholders.contains(&placeholder) {
                placeholders.push(placeholder);
            }
        }
    }
    placeholders
}

fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.into_iter().map(ToString::to_string).collect()
}
