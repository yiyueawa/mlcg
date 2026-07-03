use std::collections::{HashMap, HashSet};

use crate::manifest::{Instruction, Manifest};

pub fn validate_generated_rust_api_symbols(manifest: &Manifest) -> Result<(), String> {
    validate_item_symbols(manifest)?;
    validate_processor_methods(manifest)?;
    validate_value_methods(manifest)?;

    for instruction in &manifest.instructions {
        validate_instruction_symbols(instruction)?;
    }

    Ok(())
}

fn validate_item_symbols(manifest: &Manifest) -> Result<(), String> {
    let mut seen = HashMap::new();

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

fn validate_processor_methods(manifest: &Manifest) -> Result<(), String> {
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

fn validate_value_methods(manifest: &Manifest) -> Result<(), String> {
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

fn validate_instruction_symbols(instruction: &Instruction) -> Result<(), String> {
    validate_unique_instruction_names("instruction field", instruction, placeholders(instruction))?;
    validate_unique_instruction_names(
        "processor auto parameter",
        instruction,
        auto_processor_params(instruction)
            .into_iter()
            .map(|param| safe_ident(&param)),
    )?;
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

    if instruction.outputs.len() > 1 {
        validate_unique_instruction_names(
            "output field",
            instruction,
            instruction.outputs.iter().map(|output| safe_ident(output)),
        )?;
    }

    Ok(())
}

fn record_manifest_symbol(
    seen: &mut HashMap<String, String>,
    kind: &str,
    name: String,
    instruction: &Instruction,
) -> Result<(), String> {
    if let Some(previous) = seen.insert(name.clone(), instruction.rust_name.clone()) {
        return Err(format!(
            "generated {kind} `{name}` for instruction `{}` collides with instruction `{previous}`",
            instruction.rust_name
        ));
    }

    Ok(())
}

fn validate_unique_instruction_names(
    kind: &str,
    instruction: &Instruction,
    names: impl IntoIterator<Item = String>,
) -> Result<(), String> {
    let mut seen = HashSet::new();

    for name in names {
        if !seen.insert(name.clone()) {
            return Err(format!(
                "generated {kind} `{name}` appears more than once in instruction `{}`",
                instruction.rust_name
            ));
        }
    }

    Ok(())
}

fn explicit_value_params(instruction: &Instruction) -> Vec<String> {
    let mut params = Vec::new();
    params.extend(instruction.outputs.iter().cloned());
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
    params.extend(instruction.outputs.iter().cloned());
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
    out
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
