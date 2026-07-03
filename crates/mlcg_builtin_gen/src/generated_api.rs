use std::collections::{HashMap, HashSet};

use crate::{
    error::GenerateError,
    manifest::{Instruction, Manifest},
};

const RESERVED_OWNER_PREFIX: &str = "\0reserved:";
const RESERVED_ITEM_SYMBOLS: &[&str] = &["Arg", "OutputArg", "LabelArg"];

pub fn validate_generated_rust_api_symbols(manifest: &Manifest) -> Result<(), GenerateError> {
    validate_instruction_names(manifest)?;
    validate_parameter_names(manifest)?;
    validate_item_symbols(manifest)?;
    validate_processor_methods(manifest)?;
    validate_value_methods(manifest)?;

    for instruction in &manifest.instructions {
        validate_instruction_symbols(instruction)?;
    }

    Ok(())
}

fn validate_instruction_names(manifest: &Manifest) -> Result<(), GenerateError> {
    for instruction in &manifest.instructions {
        if instruction.rust_name.is_empty() {
            return Err(GenerateError::GeneratedApi {
                message: "instruction has empty rust_name".to_string(),
            });
        }
    }

    Ok(())
}

fn validate_parameter_names(manifest: &Manifest) -> Result<(), GenerateError> {
    for instruction in &manifest.instructions {
        validate_named_parameters(instruction, "input", &instruction.inputs)?;
        validate_named_parameters(instruction, "output", &instruction.outputs)?;
        validate_named_parameters(instruction, "label", &instruction.labels)?;
    }

    Ok(())
}

fn validate_named_parameters(
    instruction: &Instruction,
    kind: &str,
    names: &[String],
) -> Result<(), GenerateError> {
    if names.iter().any(String::is_empty) {
        return Err(GenerateError::GeneratedApi {
            message: format!(
                "instruction `{}` has empty {kind} parameter name",
                instruction.rust_name
            ),
        });
    }

    Ok(())
}

fn validate_item_symbols(manifest: &Manifest) -> Result<(), GenerateError> {
    let mut seen = HashMap::new();
    for symbol in RESERVED_ITEM_SYMBOLS {
        reserve_manifest_symbol(&mut seen, symbol);
    }

    for instruction in &manifest.instructions {
        record_manifest_symbol(&mut seen, "item", struct_name(instruction), instruction)?;
        record_manifest_symbol(
            &mut seen,
            "item",
            processor_trait_name(instruction),
            instruction,
        )?;

        if !instruction.receiver.is_empty() {
            record_manifest_symbol(
                &mut seen,
                "item",
                value_trait_name(instruction),
                instruction,
            )?;
        }

        if instruction.outputs.len() > 1 {
            record_manifest_symbol(
                &mut seen,
                "item",
                output_struct_name(instruction),
                instruction,
            )?;
        }
    }

    Ok(())
}

fn reserve_manifest_symbol(seen: &mut HashMap<String, String>, name: &str) {
    seen.insert(name.to_string(), format!("{RESERVED_OWNER_PREFIX}{name}"));
}

fn validate_processor_methods(manifest: &Manifest) -> Result<(), GenerateError> {
    let mut seen = HashMap::new();

    for instruction in &manifest.instructions {
        record_manifest_symbol(
            &mut seen,
            "processor method",
            safe_ident(&instruction.rust_name),
            instruction,
        )?;

        if !instruction.outputs.is_empty() {
            record_manifest_symbol(
                &mut seen,
                "processor method",
                safe_ident(&format!("{}_into", instruction.rust_name)),
                instruction,
            )?;
        }
    }

    Ok(())
}

fn validate_value_methods(manifest: &Manifest) -> Result<(), GenerateError> {
    let mut seen = HashMap::new();

    for instruction in manifest
        .instructions
        .iter()
        .filter(|instruction| !instruction.receiver.is_empty())
    {
        record_manifest_symbol(
            &mut seen,
            "value method",
            safe_ident(&instruction.rust_name),
            instruction,
        )?;

        if !instruction.outputs.is_empty() {
            record_manifest_symbol(
                &mut seen,
                "value method",
                safe_ident(&format!("{}_into", instruction.rust_name)),
                instruction,
            )?;
        }
    }

    Ok(())
}

fn validate_instruction_symbols(instruction: &Instruction) -> Result<(), GenerateError> {
    validate_unique_instruction_names("instruction field", instruction, placeholders(instruction))?;
    validate_unique_instruction_names(
        "processor auto parameter",
        instruction,
        auto_processor_params(instruction)
            .into_iter()
            .map(|param| safe_ident(&param)),
    )?;

    if instruction.outputs.len() > 1 {
        validate_unique_instruction_names(
            "output field",
            instruction,
            instruction.outputs.iter().map(|output| safe_ident(output)),
        )?;
    }

    validate_unique_instruction_names(
        "processor explicit parameter",
        instruction,
        explicit_processor_params(instruction)
            .into_iter()
            .map(|param| safe_ident(&param)),
    )?;

    if !instruction.receiver.is_empty() {
        validate_unique_instruction_names(
            "value parameter",
            instruction,
            value_params(instruction)
                .into_iter()
                .map(|param| safe_ident(&param)),
        )?;
        validate_unique_instruction_names(
            "value explicit parameter",
            instruction,
            explicit_value_params(instruction)
                .into_iter()
                .map(|param| safe_ident(&param)),
        )?;
    }

    validate_placeholder_roles(instruction)?;

    Ok(())
}

fn validate_placeholder_roles(instruction: &Instruction) -> Result<(), GenerateError> {
    let placeholders = emit_placeholders(&instruction.emit);
    let mut roles = Vec::new();
    if !instruction.receiver.is_empty() {
        roles.push(instruction.receiver.as_str());
    }
    roles.extend(instruction.inputs.iter().map(String::as_str));
    roles.extend(instruction.outputs.iter().map(String::as_str));
    roles.extend(instruction.labels.iter().map(String::as_str));

    let mut seen_roles = HashSet::new();
    for role in &roles {
        if !seen_roles.insert(*role) {
            return Err(GenerateError::GeneratedApi {
                message: format!(
                    "instruction `{}` classifies parameter `{role}` more than once",
                    instruction.rust_name
                ),
            });
        }
    }

    for placeholder in placeholders {
        if !roles.iter().any(|role| role == &placeholder) {
            return Err(GenerateError::GeneratedApi {
                message: format!(
                    "instruction `{}` emits placeholder `${placeholder}` that is not classified as receiver, input, output, or label",
                    instruction.rust_name
                ),
            });
        }
    }

    for role in roles {
        if !instruction
            .emit
            .iter()
            .any(|token| token == &format!("${role}"))
        {
            return Err(GenerateError::GeneratedApi {
                message: format!(
                    "instruction `{}` classifies non-emitted parameter `{role}`",
                    instruction.rust_name
                ),
            });
        }
    }

    Ok(())
}

fn record_manifest_symbol(
    seen: &mut HashMap<String, String>,
    kind: &str,
    name: String,
    instruction: &Instruction,
) -> Result<(), GenerateError> {
    if let Some(previous) = seen.insert(name.clone(), instruction.rust_name.clone()) {
        if let Some(reserved) = previous.strip_prefix(RESERVED_OWNER_PREFIX) {
            return Err(GenerateError::GeneratedApi {
                message: format!(
                    "generated {kind} `{name}` for instruction `{}` collides with reserved generated helper `{reserved}`",
                    instruction.rust_name
                ),
            });
        }
        return Err(GenerateError::GeneratedApi {
            message: format!(
                "generated {kind} `{name}` for instruction `{}` collides with instruction `{previous}`",
                instruction.rust_name
            ),
        });
    }

    Ok(())
}

fn validate_unique_instruction_names(
    kind: &str,
    instruction: &Instruction,
    names: impl IntoIterator<Item = String>,
) -> Result<(), GenerateError> {
    let mut seen = HashSet::new();

    for name in names {
        if !seen.insert(name.clone()) {
            return Err(GenerateError::GeneratedApi {
                message: format!(
                    "generated {kind} `{name}` appears more than once in instruction `{}`",
                    instruction.rust_name
                ),
            });
        }
    }

    Ok(())
}

fn explicit_value_params(instruction: &Instruction) -> Vec<String> {
    let mut params = Vec::new();
    if instruction.outputs.len() <= 1 {
        params.extend(instruction.outputs.iter().cloned());
    }
    for label in &instruction.labels {
        if !params.contains(label) {
            params.push(label.clone());
        }
    }
    for input in &instruction.inputs {
        if !params.contains(input) {
            params.push(input.clone());
        }
    }
    params
}

fn value_params(instruction: &Instruction) -> Vec<String> {
    let mut params = Vec::new();
    for label in &instruction.labels {
        if !params.contains(label) {
            params.push(label.clone());
        }
    }
    for input in &instruction.inputs {
        if !params.contains(input) {
            params.push(input.clone());
        }
    }
    params
}

fn auto_processor_params(instruction: &Instruction) -> Vec<String> {
    let mut params = Vec::new();
    if !instruction.receiver.is_empty() {
        params.push(instruction.receiver.clone());
    }
    for label in &instruction.labels {
        if !params.contains(label) {
            params.push(label.clone());
        }
    }
    for input in &instruction.inputs {
        if !params.contains(input) {
            params.push(input.clone());
        }
    }
    params
}

fn explicit_processor_params(instruction: &Instruction) -> Vec<String> {
    let mut params = Vec::new();
    if instruction.outputs.len() <= 1 {
        params.extend(instruction.outputs.iter().cloned());
    }
    if !instruction.receiver.is_empty() && !params.contains(&instruction.receiver) {
        params.push(instruction.receiver.clone());
    }
    for label in &instruction.labels {
        if !params.contains(label) {
            params.push(label.clone());
        }
    }
    for input in &instruction.inputs {
        if !params.contains(input) {
            params.push(input.clone());
        }
    }
    params
}

fn placeholders(instruction: &Instruction) -> Vec<String> {
    let mut names = Vec::new();
    for token in &instruction.emit {
        if let Some(name) = token.strip_prefix('$') {
            if !names.iter().any(|existing| existing == name) {
                names.push(safe_ident(name));
            }
        }
    }
    names
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

fn struct_name(instruction: &Instruction) -> String {
    to_pascal_string(&instruction.rust_name)
}

fn output_struct_name(instruction: &Instruction) -> String {
    format!("{}Output", struct_name(instruction))
}

fn processor_trait_name(instruction: &Instruction) -> String {
    format!("Processor{}Ext", struct_name(instruction))
}

fn value_trait_name(instruction: &Instruction) -> String {
    format!("Value{}Ext", struct_name(instruction))
}

fn to_pascal_string(name: &str) -> String {
    let mut out = String::new();
    for part in name.split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if is_rust_keyword(&out) {
        format!("Arg{out}")
    } else {
        out
    }
}

fn safe_ident(name: &str) -> String {
    if is_rust_keyword(name) {
        format!("arg_{name}")
    } else {
        name.to_string()
    }
}

fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "Self"
            | "as"
            | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "union"
    )
}
