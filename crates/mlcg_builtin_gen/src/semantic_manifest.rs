use crate::{
    error::GenerateError,
    manifest::{Instruction, Manifest},
    raw_statement::{RawEnum, RawField, RawStatement, RawStatementManifest},
};

#[derive(Debug, Clone)]
struct EnumSelection {
    field: String,
    variant: String,
    arity: Option<usize>,
}

pub fn derive_semantic_manifest(raw: &RawStatementManifest) -> Manifest {
    try_derive_semantic_manifest(raw).expect("raw manifest selector enums are defined")
}

pub fn try_derive_semantic_manifest(raw: &RawStatementManifest) -> Result<Manifest, GenerateError> {
    validate_selector_enums(raw)?;

    Ok(Manifest {
        version: raw.version.clone(),
        instructions: raw
            .statements
            .iter()
            .flat_map(|statement| derive_statement_instructions(statement, &raw.enums))
            .collect(),
    })
}

fn validate_selector_enums(raw: &RawStatementManifest) -> Result<(), GenerateError> {
    for statement in &raw.statements {
        for field in &statement.fields {
            if is_ignored_field(statement, &field.name) || !is_selector_field(&field.name) {
                continue;
            }
            if is_basic_field_type(&field.ty) {
                continue;
            }
            if enum_variants(&raw.enums, &field.ty).is_none() {
                return Err(GenerateError::RequiredSourceItemMissing {
                    item: format!("enum {}", field.ty),
                });
            }
        }
    }

    Ok(())
}

fn is_basic_field_type(ty: &str) -> bool {
    matches!(ty, "String" | "int" | "boolean" | "float" | "double")
}

fn derive_statement_instructions(statement: &RawStatement, enums: &[RawEnum]) -> Vec<Instruction> {
    let enum_fields: Vec<_> = statement
        .fields
        .iter()
        .filter(|field| !is_ignored_field(statement, &field.name))
        .filter(|field| is_selector_field(&field.name))
        .filter_map(|field| enum_variants(enums, &field.ty).map(|raw_enum| (field, raw_enum)))
        .collect();

    if enum_fields.is_empty() {
        return vec![derive_instruction(statement, &[])];
    }

    let mut selections = Vec::new();
    let mut out = Vec::new();
    expand_enum_selections(statement, &enum_fields, 0, &mut selections, &mut out);
    out
}

fn expand_enum_selections(
    statement: &RawStatement,
    enum_fields: &[(&RawField, &RawEnum)],
    index: usize,
    selections: &mut Vec<EnumSelection>,
    out: &mut Vec<Instruction>,
) {
    if index == enum_fields.len() {
        out.push(derive_instruction(statement, selections));
        return;
    }

    let (field, raw_enum) = enum_fields[index];
    for variant in &raw_enum.variants {
        selections.push(EnumSelection {
            field: field.name.clone(),
            variant: variant.clone(),
            arity: raw_enum.arities.get(variant).copied(),
        });
        expand_enum_selections(statement, enum_fields, index + 1, selections, out);
        selections.pop();
    }
}

fn derive_instruction(statement: &RawStatement, enum_selections: &[EnumSelection]) -> Instruction {
    let output_names = output_fields(statement);
    let labels = label_fields(statement);
    let operand_limit = operand_arity(enum_selections);
    let receiver = receiver_field(statement, &output_names, enum_selections, operand_limit)
        .unwrap_or_default();
    let unused_operands =
        unused_operand_fields(statement, &output_names, enum_selections, operand_limit);
    let inputs = statement
        .fields
        .iter()
        .filter(|field| !is_enum_selection(&field.name, enum_selections))
        .filter(|field| !is_ignored_field(statement, &field.name))
        .filter(|field| !output_names.iter().any(|output| output == &field.name))
        .filter(|field| !labels.iter().any(|label| label == &field.name))
        .filter(|field| receiver != field.name)
        .filter(|field| !unused_operands.iter().any(|unused| unused == &field.name))
        .map(|field| field.name.clone())
        .collect();

    Instruction {
        family: statement.name.clone(),
        variant: variant_name(statement, enum_selections),
        rust_name: rust_name(statement, enum_selections),
        emit: emit_tokens(statement, enum_selections, &unused_operands),
        receiver,
        inputs,
        outputs: output_names,
        labels,
    }
}

fn emit_tokens(
    statement: &RawStatement,
    enum_selections: &[EnumSelection],
    unused_operands: &[String],
) -> Vec<String> {
    let mut emit = Vec::with_capacity(statement.fields.len() + 1);
    emit.push(statement.name.clone());
    emit.extend(statement.fields.iter().map(|field| {
        enum_selections
            .iter()
            .find_map(|selection| {
                (selection.field == field.name).then(|| selection.variant.clone())
            })
            .unwrap_or_else(|| {
                if is_ignored_field(statement, &field.name) {
                    default_emit_token(field)
                } else if unused_operands.iter().any(|unused| unused == &field.name) {
                    "0".to_string()
                } else {
                    format!("${}", field.name)
                }
            })
    }));
    emit
}

fn label_fields(statement: &RawStatement) -> Vec<String> {
    statement
        .fields
        .iter()
        .filter(|field| !is_ignored_field(statement, &field.name))
        .filter(|field| is_label_field(&field.name))
        .map(|field| field.name.clone())
        .collect()
}

fn output_fields(statement: &RawStatement) -> Vec<String> {
    let outputs: Vec<_> = statement
        .fields
        .iter()
        .filter(|field| !is_ignored_field(statement, &field.name))
        .filter(|field| is_output_field(statement, &field.name))
        .map(|field| field.name.clone())
        .collect();
    outputs
}

fn receiver_field(
    statement: &RawStatement,
    outputs: &[String],
    enum_selections: &[EnumSelection],
    operand_limit: Option<usize>,
) -> Option<String> {
    if operand_limit == Some(0) {
        return None;
    }

    let operands = if operand_limit.is_some() {
        arity_limited_operand_fields(statement, outputs, enum_selections)
    } else {
        operand_fields(statement, outputs, enum_selections)
    };
    let preferred = receiver_priority().iter().find_map(|preferred| {
        operands
            .iter()
            .find(|operand| operand.as_str() == *preferred)
            .cloned()
    });
    let ignored_preferred = receiver_priority()
        .iter()
        .any(|preferred| is_ignored_field(statement, preferred));
    if preferred.is_none() && ignored_preferred {
        return None;
    }
    if preferred.is_some() || outputs.len() <= 1 {
        preferred.or_else(|| operands.into_iter().next())
    } else {
        None
    }
}

fn unused_operand_fields(
    statement: &RawStatement,
    outputs: &[String],
    enum_selections: &[EnumSelection],
    operand_limit: Option<usize>,
) -> Vec<String> {
    let Some(limit) = operand_limit else {
        return Vec::new();
    };
    arity_limited_operand_fields(statement, outputs, enum_selections)
        .into_iter()
        .skip(limit)
        .collect()
}

fn arity_limited_operand_fields(
    statement: &RawStatement,
    outputs: &[String],
    enum_selections: &[EnumSelection],
) -> Vec<String> {
    let operands = operand_fields(statement, outputs, enum_selections);
    let comparison_operands: Vec<_> = operands
        .iter()
        .filter(|operand| is_comparison_operand(operand))
        .cloned()
        .collect();

    if comparison_operands.is_empty() {
        operands
    } else {
        comparison_operands
    }
}

fn operand_fields(
    statement: &RawStatement,
    outputs: &[String],
    enum_selections: &[EnumSelection],
) -> Vec<String> {
    statement
        .fields
        .iter()
        .filter(|field| field.ty == "String")
        .filter(|field| !is_ignored_field(statement, &field.name))
        .filter(|field| !outputs.iter().any(|output| output == &field.name))
        .filter(|field| !is_enum_selection(&field.name, enum_selections))
        .map(|field| field.name.clone())
        .collect()
}

fn is_comparison_operand(name: &str) -> bool {
    matches!(name, "value" | "compare" | "comp0" | "comp1")
}

fn is_ignored_field(statement: &RawStatement, name: &str) -> bool {
    statement
        .ignored_fields
        .iter()
        .any(|ignored| ignored == name)
}

fn default_emit_token(field: &RawField) -> String {
    let Some(default) = field.default.as_deref() else {
        return "0".to_string();
    };
    default
        .rsplit_once('.')
        .map_or(default, |(_, variant)| variant)
        .to_string()
}

fn operand_arity(enum_selections: &[EnumSelection]) -> Option<usize> {
    enum_selections.iter().find_map(|selection| selection.arity)
}

fn is_label_field(name: &str) -> bool {
    name == "destIndex"
}

fn is_output_field(statement: &RawStatement, name: &str) -> bool {
    matches!(name, "output" | "result" | "dest")
        || is_to_output(statement, name)
        || name
            .strip_prefix("out")
            .and_then(|suffix| suffix.chars().next())
            .is_some_and(char::is_uppercase)
}

fn is_to_output(statement: &RawStatement, name: &str) -> bool {
    name == "to"
        && statement.fields.len() >= 3
        && statement.fields.iter().any(|field| field.name == "from")
}

fn receiver_priority() -> &'static [&'static str] {
    &["target", "of", "unit", "radar", "to", "from"]
}

fn is_selector_field(name: &str) -> bool {
    matches!(
        name,
        "op" | "type" | "rule" | "action" | "layer" | "shape" | "locate"
    )
}

fn enum_variants<'a>(enums: &'a [RawEnum], ty: &str) -> Option<&'a RawEnum> {
    enums.iter().find(|raw_enum| raw_enum.name == ty)
}

fn is_enum_selection(name: &str, enum_selections: &[EnumSelection]) -> bool {
    enum_selections
        .iter()
        .any(|selection| selection.field == name)
}

fn variant_name(statement: &RawStatement, enum_selections: &[EnumSelection]) -> String {
    if enum_selections.is_empty() {
        statement.name.clone()
    } else {
        enum_selections
            .iter()
            .map(|selection| selection.variant.clone())
            .collect::<Vec<_>>()
            .join("_")
    }
}

fn rust_name(statement: &RawStatement, enum_selections: &[EnumSelection]) -> String {
    let mut name = sanitize_name(&statement.name);
    for selection in enum_selections {
        name.push('_');
        name.push_str(&sanitize_name(&selection.variant));
    }
    name
}

fn sanitize_name(name: &str) -> String {
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
