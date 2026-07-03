use std::fs;

use mlcg_builtin_gen::manifest::Manifest;

#[test]
fn v158_1_semantic_manifest_classifies_every_emit_placeholder() {
    let manifest_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../mlcg_builtin/manifests/v158_1.toml"
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(manifest_path).expect("read v158.1 manifest"))
            .expect("parse v158.1 manifest");

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
