use std::collections::{BTreeMap, BTreeSet};

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ignored_fields: Vec<String>,
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
    let classes = parse_classes(source)?;
    let mut statements = Vec::new();
    let mut offset = 0;
    let marker = "@RegisterStatement(\"";

    while let Some(marker_start) = find_code(source, marker, offset) {
        let name_start = marker_start + marker.len();
        let name_end = source[name_start..]
            .find("\"")
            .map(|index| name_start + index)
            .context(RequiredItemMissingSnafu {
                item: "register statement name terminator",
            })?;
        let name = &source[name_start..name_end];

        let class_search_start = name_end + 1;
        let class_marker = "public static class ";
        let class_start = find_code(source, class_marker, class_search_start)
            .map(|index| index + class_marker.len())
            .context(RequiredItemMissingSnafu {
                item: "registered statement class declaration",
            })?;
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

        let fields = parse_fields_with_superclasses(&classes, class, body);
        statements.push(RawStatement {
            name: name.to_string(),
            class: class.to_string(),
            instruction: parse_instruction(body),
            category: parse_category(body),
            ignored_fields: parse_ignored_fields(body, &fields),
            fields,
        });

        offset = brace_end + 1;
    }

    Ok(RawStatementManifest {
        version: version.to_string(),
        statements,
        enums: Vec::new(),
    })
}

#[derive(Debug)]
struct ClassInfo {
    superclass: Option<String>,
    body: String,
}

fn parse_classes(source: &str) -> Result<BTreeMap<String, ClassInfo>, GenerateError> {
    let mut classes = BTreeMap::new();
    let mut offset = 0;
    let marker = "public static class ";

    while let Some(marker_start) = find_code(source, marker, offset) {
        let class_start = marker_start + marker.len();
        let class_end = source[class_start..]
            .find(|c: char| c.is_whitespace() || c == '{')
            .map(|index| class_start + index)
            .context(RequiredItemMissingSnafu { item: "class name" })?;
        let class = source[class_start..class_end].trim().to_string();
        let brace_start = source[class_end..]
            .find('{')
            .map(|index| class_end + index)
            .context(RequiredItemMissingSnafu { item: "class body" })?;
        let superclass = parse_extends(&source[class_end..brace_start]);
        let brace_end = matching_brace(source, brace_start).context(RequiredItemMissingSnafu {
            item: "class closing brace",
        })?;
        let body = source[brace_start + 1..brace_end].to_string();

        classes.insert(class, ClassInfo { superclass, body });
        offset = brace_end + 1;
    }

    Ok(classes)
}

fn parse_extends(header: &str) -> Option<String> {
    let marker = "extends ";
    let start = header.find(marker)? + marker.len();
    let rest = header[start..].trim_start();
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    (!rest[..end].is_empty()).then(|| rest[..end].to_string())
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
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
                continue;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
                continue;
            }
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

fn find_code(source: &str, marker: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let marker = marker.as_bytes();
    let mut index = start;

    while index < bytes.len() {
        if bytes[index..].starts_with(marker) {
            return Some(index);
        }

        match bytes[index] {
            b'"' => index = skip_string(bytes, index) + 1,
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                index = skip_line_comment(bytes, index + 2);
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                index = skip_block_comment(bytes, index + 2);
            }
            _ => index += 1,
        }
    }

    None
}

fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    while index < bytes.len() && bytes[index] != b'\n' {
        index += 1;
    }
    (index + 1).min(bytes.len())
}

fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut index = start;
    while index + 1 < bytes.len() {
        if bytes[index] == b'*' && bytes[index + 1] == b'/' {
            return index + 2;
        }
        index += 1;
    }
    bytes.len()
}

fn parse_instruction(body: &str) -> Option<String> {
    let body = parse_linstruction_build_body(body)?;
    let marker = "return new ";
    let start = body.find(marker)? + marker.len();
    let end = body[start..].find('(')? + start;
    Some(body[start..end].trim().to_string())
}

fn parse_category(body: &str) -> Option<String> {
    let body = parse_method_body(body, "LCategory", "category")?;
    let marker = "return LCategory.";
    let start = body.find(marker)? + marker.len();
    let end = body[start..].find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))? + start;
    Some(body[start..end].to_string())
}

fn parse_fields(body: &str) -> Vec<RawField> {
    let mut fields = Vec::new();
    for statement in top_level_semicolon_statements(body) {
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
        for declarator in comma_separated_top_level(rest) {
            if let Some(field) = parse_declarator(ty, declarator) {
                fields.push(field);
            }
        }
    }
    fields
}

fn comma_separated_top_level(source: &str) -> Vec<&str> {
    separated_top_level(source, b',', true)
}

fn top_level_semicolon_statements(source: &str) -> Vec<&str> {
    separated_top_level(source, b';', false)
}

fn separated_top_level(source: &str, separator: u8, include_tail: bool) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut statements = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    start = index + 1;
                }
            }
            byte if byte == separator && depth == 0 => {
                statements.push(&source[start..index]);
                start = index + 1;
            }
            b'"' => index = skip_string(bytes, index),
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let next = skip_line_comment(bytes, index + 2);
                if source[start..index].trim().is_empty() {
                    start = next;
                }
                index = next;
                continue;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let next = skip_block_comment(bytes, index + 2);
                if source[start..index].trim().is_empty() {
                    start = next;
                }
                index = next;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    if include_tail && start < source.len() {
        statements.push(&source[start..]);
    }

    statements
}

fn parse_fields_with_superclasses(
    classes: &BTreeMap<String, ClassInfo>,
    class: &str,
    body: &str,
) -> Vec<RawField> {
    let mut fields = parse_fields(body);
    fields.extend(superclass_fields(classes, class));
    apply_constructor_defaults(class, body, &mut fields);
    remove_hidden_fields(&mut fields);
    fields
}

fn superclass_fields(classes: &BTreeMap<String, ClassInfo>, class: &str) -> Vec<RawField> {
    let Some(superclass_name) = classes
        .get(class)
        .and_then(|class_info| class_info.superclass.as_deref())
    else {
        return Vec::new();
    };
    let Some(superclass) = classes.get(superclass_name) else {
        return Vec::new();
    };

    let mut fields = parse_fields(&superclass.body);
    fields.extend(superclass_fields(classes, superclass_name));
    apply_constructor_defaults(superclass_name, &superclass.body, &mut fields);
    fields
}

fn apply_constructor_defaults(class: &str, body: &str, fields: &mut [RawField]) {
    let assignments = parse_constructor_assignments(class, body);
    for field in fields {
        if let Some(default) = assignments.get(&field.name) {
            field.default = Some(default.clone());
        }
    }
}

fn remove_hidden_fields(fields: &mut Vec<RawField>) {
    let mut seen = BTreeSet::new();
    fields.retain(|field| seen.insert(field.name.clone()));
}

fn parse_constructor_assignments(class: &str, body: &str) -> BTreeMap<String, String> {
    let mut assignments = BTreeMap::new();
    let marker = format!("public {class}(");
    let Some(start) = body.find(&marker) else {
        return assignments;
    };
    let Some(brace_start) = body[start..].find('{').map(|index| start + index) else {
        return assignments;
    };
    let Some(brace_end) = matching_brace(body, brace_start) else {
        return assignments;
    };
    let constructor_body = &body[brace_start + 1..brace_end];

    for statement in top_level_semicolon_statements(constructor_body) {
        let Some((left, right)) = statement.split_once('=') else {
            continue;
        };
        let name = left.trim();
        if is_simple_identifier(name) {
            assignments.insert(name.to_string(), clean_default(right.trim()));
        }
    }

    assignments
}

fn parse_ignored_fields(body: &str, fields: &[RawField]) -> Vec<String> {
    let Some(build_body) = parse_linstruction_build_body(body) else {
        return Vec::new();
    };
    fields
        .iter()
        .filter(|field| !contains_identifier(build_body, &field.name))
        .map(|field| field.name.clone())
        .collect()
}

fn parse_linstruction_build_body(body: &str) -> Option<&str> {
    parse_method_body(body, "LInstruction", "build")
}

fn parse_method_body<'a>(body: &'a str, return_type: &str, method_name: &str) -> Option<&'a str> {
    let mut offset = 0;
    while let Some(start) = find_code(body, return_type, offset) {
        let after_return_type = start + return_type.len();
        if !is_identifier_boundary(body, start, after_return_type) {
            offset = after_return_type;
            continue;
        }
        let after_spaces = skip_whitespace(body, after_return_type);
        let method_end = after_spaces + method_name.len();
        if body[after_spaces..].starts_with(method_name)
            && is_identifier_boundary(body, after_spaces, method_end)
            && body[method_end..].trim_start().starts_with('(')
        {
            let brace_start = body[method_end..]
                .find('{')
                .map(|index| method_end + index)?;
            let brace_end = matching_brace(body, brace_start)?;
            return Some(&body[brace_start + 1..brace_end]);
        }
        offset = after_return_type;
    }
    None
}

fn skip_whitespace(source: &str, start: usize) -> usize {
    source[start..]
        .find(|ch: char| !ch.is_whitespace())
        .map_or(source.len(), |index| start + index)
}

fn is_identifier_boundary(source: &str, start: usize, end: usize) -> bool {
    let before = source[..start].chars().next_back();
    let after = source[end..].chars().next();
    !before.is_some_and(is_ident_char) && !after.is_some_and(is_ident_char)
}

fn contains_identifier(source: &str, ident: &str) -> bool {
    let bytes = source.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        if bytes[offset] == b'"' {
            offset = skip_string(bytes, offset) + 1;
            continue;
        }
        let end = offset + ident.len();
        if end <= bytes.len()
            && &source[offset..end] == ident
            && is_identifier_boundary(source, offset, end)
        {
            return true;
        }
        offset += 1;
    }
    false
}

fn is_simple_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(is_ident_char)
}

fn is_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
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
    if lambda_body_is_constant_true(&token[arrow + "->".len()..]) {
        return Some(0);
    }

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

fn lambda_body_is_constant_true(body: &str) -> bool {
    let Some(rest) = body.trim_start().strip_prefix("true") else {
        return false;
    };
    rest.trim_start()
        .chars()
        .next()
        .is_none_or(|ch| matches!(ch, ')' | ',' | ';'))
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
