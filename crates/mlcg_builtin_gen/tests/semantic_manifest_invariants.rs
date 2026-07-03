use std::fs;

use mlcg_builtin_gen::{
    generated_api::validate_generated_rust_api_symbols,
    manifest::{Instruction, Manifest},
    raw_statement::RawStatementManifest,
    semantic_manifest::derive_semantic_manifest,
};

#[test]
fn v158_1_semantic_manifest_matches_raw_manifest_derivation() {
    let raw = read_v158_1_raw_manifest();
    let derived = derive_semantic_manifest(&raw);
    let committed = read_v158_1_manifest();

    assert_eq!(derived, committed);
}

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
        error.to_string(),
        "generated processor method `foo_into` for instruction `foo_into` collides with instruction `foo`"
    );
}

#[test]
fn generated_rust_api_symbol_validation_rejects_helper_item_collision() {
    let manifest = Manifest {
        version: "fixture".to_string(),
        instructions: vec![Instruction {
            family: "fixture".to_string(),
            variant: "arg".to_string(),
            rust_name: "arg".to_string(),
            emit: strings(["arg"]),
            receiver: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            labels: Vec::new(),
        }],
    };

    let error = validate_generated_rust_api_symbols(&manifest)
        .expect_err("generated helper item collision is rejected");

    assert_eq!(
        error.to_string(),
        "generated item `Arg` for instruction `arg` collides with reserved generated helper `Arg`"
    );
}

#[test]
fn generated_rust_api_symbol_validation_reports_multi_output_field_collision_as_output_field() {
    let manifest = Manifest {
        version: "fixture".to_string(),
        instructions: vec![Instruction {
            family: "fixture".to_string(),
            variant: "multi".to_string(),
            rust_name: "multi".to_string(),
            emit: strings(["multi"]),
            receiver: String::new(),
            inputs: Vec::new(),
            outputs: strings(["type", "arg_type"]),
            labels: Vec::new(),
        }],
    };

    let error = validate_generated_rust_api_symbols(&manifest)
        .expect_err("generated output struct field collision is rejected");

    assert_eq!(
        error.to_string(),
        "generated output field `arg_type` appears more than once in instruction `multi`"
    );
}

#[test]
fn generated_rust_api_symbol_validation_rejects_keyword_item_collision_after_escaping() {
    let manifest = Manifest {
        version: "fixture".to_string(),
        instructions: vec![
            Instruction {
                family: "fixture".to_string(),
                variant: "self".to_string(),
                rust_name: "self".to_string(),
                emit: strings(["self"]),
                receiver: String::new(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                labels: Vec::new(),
            },
            Instruction {
                family: "fixture".to_string(),
                variant: "arg_self".to_string(),
                rust_name: "arg_self".to_string(),
                emit: strings(["arg_self"]),
                receiver: String::new(),
                inputs: Vec::new(),
                outputs: Vec::new(),
                labels: Vec::new(),
            },
        ],
    };

    let error = validate_generated_rust_api_symbols(&manifest)
        .expect_err("escaped keyword item collision is rejected");

    assert_eq!(
        error.to_string(),
        "generated item `ArgSelf` for instruction `arg_self` collides with instruction `self`"
    );
}

#[test]
fn generated_rust_api_symbol_validation_rejects_unclassified_emit_placeholder() {
    let manifest = Manifest {
        version: "fixture".to_string(),
        instructions: vec![Instruction {
            family: "fixture".to_string(),
            variant: "bad".to_string(),
            rust_name: "bad".to_string(),
            emit: strings(["bad", "$missing"]),
            receiver: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            labels: Vec::new(),
        }],
    };

    let error = validate_generated_rust_api_symbols(&manifest)
        .expect_err("unclassified emit placeholder is rejected");

    assert_eq!(
        error.to_string(),
        "instruction `bad` emits placeholder `$missing` that is not classified as receiver, input, output, or label"
    );
}

#[test]
fn generated_rust_api_symbol_validation_rejects_non_emitted_classified_parameter() {
    let manifest = Manifest {
        version: "fixture".to_string(),
        instructions: vec![Instruction {
            family: "fixture".to_string(),
            variant: "bad".to_string(),
            rust_name: "bad".to_string(),
            emit: strings(["bad"]),
            receiver: String::new(),
            inputs: strings(["unused"]),
            outputs: Vec::new(),
            labels: Vec::new(),
        }],
    };

    let error = validate_generated_rust_api_symbols(&manifest)
        .expect_err("non-emitted classified parameter is rejected");

    assert_eq!(
        error.to_string(),
        "instruction `bad` classifies non-emitted parameter `unused`"
    );
}

#[test]
fn generated_rust_api_symbol_validation_rejects_parameter_with_multiple_roles() {
    let manifest = Manifest {
        version: "fixture".to_string(),
        instructions: vec![Instruction {
            family: "fixture".to_string(),
            variant: "bad".to_string(),
            rust_name: "bad".to_string(),
            emit: strings(["bad", "$slot"]),
            receiver: String::new(),
            inputs: strings(["slot"]),
            outputs: strings(["slot"]),
            labels: Vec::new(),
        }],
    };

    let error = validate_generated_rust_api_symbols(&manifest)
        .expect_err("parameter with multiple roles is rejected");

    assert_eq!(
        error.to_string(),
        "instruction `bad` classifies parameter `slot` more than once"
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

fn read_v158_1_raw_manifest() -> RawStatementManifest {
    let manifest_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../mlcg_builtin/manifests/raw/v158_1_statements.toml"
    );
    toml::from_str(&fs::read_to_string(manifest_path).expect("read v158.1 raw manifest"))
        .expect("parse v158.1 raw manifest")
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
