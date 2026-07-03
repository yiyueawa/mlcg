use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use snafu::OptionExt;

use crate::error::{GenerateError, RequiredItemMissingSnafu};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawStatementManifest {
    pub version: String,
    pub statements: Vec<RawStatement>,
    #[serde(default)]
    pub enums: Vec<RawEnum>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawEnum {
    pub name: String,
    pub variants: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub arities: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawStatement {
    pub name: String,
    pub class: String,
    pub instruction: Option<String>,
    pub category: Option<String>,
    pub fields: Vec<RawField>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawField {
    pub ty: String,
    pub name: String,
    pub default: Option<String>,
}

pub fn scan_raw_statements(
    version: &str,
    source: &str,
) -> Result<RawStatementManifest, GenerateError> {
    let mut statements = Vec::new();
    let mut offset = 0;
    let marker = "@RegisterStatement(\"";

    while let Some(relative) = source[offset..].find(marker) {
        let marker_start = offset + relative;
        if is_line_comment_at(source, marker_start) {
            offset = marker_start + marker.len();
            continue;
        }
        let name_start = marker_start + marker.len();
        let name_end = source[name_start..]
            .find("\"")
            .map(|index| name_start + index)
            .context(RequiredItemMissingSnafu {
                item: "register statement name terminator",
            })?;
        let name = &source[name_start..name_end];

        let class_search_start = name_end;
        let class_relative = source[class_search_start..]
            .find("public static class ")
            .context(RequiredItemMissingSnafu {
                item: "registered statement class declaration",
            })?;
        let class_start = class_search_start + class_relative + "public static class ".len();
        let class_end = source[class_start..]
            .find(|c: char| c.is_whitespace() || c == '{')
            .map(|index| class_start + index)
            .context(RequiredItemMissingSnafu {
                item: "registered statement class name",
            })?;
        let class = &source[class_start..class_end];
        let brace_start = source[class_end..]
            .find('{')
            .map(|index| class_end + index)
            .context(RequiredItemMissingSnafu {
                item: "registered statement class body",
            })?;
        let brace_end = matching_brace(source, brace_start).context(RequiredItemMissingSnafu {
            item: "registered statement class closing brace",
        })?;
        let body = &source[brace_start + 1..brace_end];

        statements.push(RawStatement {
            name: name.to_string(),
            class: class.to_string(),
            instruction: parse_instruction(body),
            category: parse_category(body),
            fields: parse_fields(body),
        });

        offset = brace_end + 1;
    }

    Ok(RawStatementManifest {
        version: version.to_string(),
        statements,
        enums: Vec::new(),
    })
}

impl RawStatementManifest {
    pub fn to_toml(&self) -> Result<String, GenerateError> {
        toml::to_string_pretty(self).map_err(|source| GenerateError::Serialize { source })
    }
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            b'"' => index = skip_string(bytes, index),
            _ => {}
        }
        index += 1;
    }
    None
}

fn skip_string(bytes: &[u8], quote: usize) -> usize {
    let mut index = quote + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += 2;
        } else if bytes[index] == b'"' {
            return index;
        } else {
            index += 1;
        }
    }
    bytes.len().saturating_sub(1)
}

fn parse_instruction(body: &str) -> Option<String> {
    let marker = "return new ";
    let start = body.find(marker)? + marker.len();
    let end = body[start..].find('(')? + start;
    Some(body[start..end].trim().to_string())
}

fn parse_category(body: &str) -> Option<String> {
    let marker = "return LCategory.";
    let start = body.find(marker)? + marker.len();
    let end = body[start..].find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))? + start;
    Some(body[start..end].to_string())
}

fn parse_fields(body: &str) -> Vec<RawField> {
    let mut fields = Vec::new();
    for statement in body.split(';') {
        let trimmed = statement.trim();
        if !trimmed.starts_with("public ") {
            continue;
        }
        if trimmed.contains(" static ") || trimmed.starts_with("public static ") {
            continue;
        }
        if trimmed.contains(" transient ") || trimmed.starts_with("public transient ") {
            continue;
        }
        if trimmed.contains('(') {
            continue;
        }

        let declaration = trimmed.trim_start_matches("public ").trim();
        let mut parts = declaration.splitn(2, char::is_whitespace);
        let Some(ty) = parts.next() else { continue };
        let Some(rest) = parts.next() else { continue };
        for declarator in rest.split(',') {
            if let Some(field) = parse_declarator(ty, declarator) {
                fields.push(field);
            }
        }
    }
    fields
}

fn parse_declarator(ty: &str, declarator: &str) -> Option<RawField> {
    let trimmed = declarator.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (name, default) = if let Some((left, right)) = trimmed.split_once('=') {
        (left.trim(), Some(clean_default(right.trim())))
    } else {
        (trimmed, None)
    };
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    Some(RawField {
        ty: ty.to_string(),
        name: name.to_string(),
        default,
    })
}

fn clean_default(value: &str) -> String {
    let value = value.trim();
    if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn is_line_comment_at(source: &str, index: usize) -> bool {
    let line_start = source[..index]
        .rfind('\n')
        .map_or(0, |position| position + 1);
    source[line_start..index].contains("//")
}

pub fn scan_raw_enum_variants(name: &str, source: &str) -> Result<RawEnum, GenerateError> {
    let enum_marker = format!("enum {name}");
    let enum_start = source
        .find(&enum_marker)
        .context(RequiredItemMissingSnafu {
            item: "enum declaration",
        })?;
    let brace_start = source[enum_start..]
        .find('{')
        .map(|index| enum_start + index)
        .context(RequiredItemMissingSnafu { item: "enum body" })?;
    let brace_end = matching_brace(source, brace_start).context(RequiredItemMissingSnafu {
        item: "enum closing brace",
    })?;
    let body = &source[brace_start + 1..brace_end];
    let constants = parse_enum_constants(body);
    Ok(RawEnum {
        name: name.to_string(),
        variants: constants
            .iter()
            .map(|constant| constant.name.clone())
            .collect(),
        arities: constants
            .into_iter()
            .filter_map(|constant| constant.arity.map(|arity| (constant.name, arity)))
            .collect(),
    })
}

#[derive(Debug)]
struct RawEnumConstant {
    name: String,
    arity: Option<usize>,
}

fn parse_enum_constants(body: &str) -> Vec<RawEnumConstant> {
    let mut variants = Vec::new();
    let mut token = String::new();
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut in_constants = true;
    let mut chars = body.chars().peekable();

    while let Some(ch) = chars.next() {
        if !in_constants {
            break;
        }
        match ch {
            '"' => {
                token.push(ch);
                let mut escaped = false;
                for next in chars.by_ref() {
                    token.push(next);
                    if escaped {
                        escaped = false;
                    } else if next == '\\' {
                        escaped = true;
                    } else if next == '"' {
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'/') => {
                for next in chars.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = '\0';
                for next in chars.by_ref() {
                    if prev == '*' && next == '/' {
                        break;
                    }
                    prev = next;
                }
            }
            '(' => {
                paren_depth += 1;
                token.push(ch);
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                token.push(ch);
            }
            '{' => {
                brace_depth += 1;
                token.push(ch);
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                token.push(ch);
            }
            ',' if paren_depth == 0 && brace_depth == 0 => {
                push_enum_variant(&mut variants, &token);
                token.clear();
            }
            ';' if paren_depth == 0 && brace_depth == 0 => {
                push_enum_variant(&mut variants, &token);
                in_constants = false;
            }
            _ => token.push(ch),
        }
    }
    if in_constants {
        push_enum_variant(&mut variants, &token);
    }
    variants
}

fn push_enum_variant(variants: &mut Vec<RawEnumConstant>, token: &str) {
    let trimmed = token.trim();
    if trimmed.is_empty() || trimmed.starts_with('@') {
        return;
    }
    let name: String = trimmed
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .collect();
    if !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
    {
        variants.push(RawEnumConstant {
            name,
            arity: infer_lambda_arity(trimmed),
        });
    }
}

fn infer_lambda_arity(token: &str) -> Option<usize> {
    let arrow = token.find("->")?;
    let before = token[..arrow].trim_end();
    if before.ends_with(')') {
        let open = matching_paren_before(before, before.len() - 1)?;
        let params = before[open + 1..before.len() - 1].trim();
        if params.is_empty() {
            Some(0)
        } else {
            Some(params.split(',').count())
        }
    } else {
        Some(1)
    }
}

fn matching_paren_before(source: &str, close: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    for index in (0..=close).rev() {
        match bytes[index] {
            b')' => depth += 1,
            b'(' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}
