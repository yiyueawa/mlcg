use std::collections::{BTreeMap, BTreeSet};

use snafu::OptionExt;

use crate::{
    error::emit_error,
    lower::{LabelTable, PartialProgram, PartialToken},
    processor::ProcessorHandle,
    EmitError, ValueId,
};

#[derive(Debug, Default)]
pub(crate) struct NameAllocator {
    names: BTreeMap<ValueId, String>,
    used: BTreeMap<String, usize>,
    allocated: BTreeSet<String>,
}

impl NameAllocator {
    pub(crate) fn name_for(&mut self, id: ValueId, hint: Option<&str>) -> String {
        if let Some(name) = self.names.get(&id) {
            return name.clone();
        }

        if let Some(explicit) = hint.filter(|value| !value.is_empty()) {
            if self.allocated.insert(explicit.to_string()) {
                self.names.insert(id, explicit.to_string());
                return explicit.to_string();
            }
        }

        let base = hint.filter(|value| !value.is_empty()).unwrap_or("__mlcg");
        let count = self.used.entry(base.to_string()).or_insert(0);
        let name = loop {
            let candidate = format!("{base}_{count}");
            *count += 1;
            if self.allocated.insert(candidate.clone()) {
                break candidate;
            }
        };
        self.names.insert(id, name.clone());
        name
    }
}

pub(crate) fn emit_partial<P: 'static>(
    partial: &PartialProgram<P>,
    handle: &ProcessorHandle<P>,
    labels: &LabelTable,
    value_names: &BTreeMap<ValueId, String>,
) -> Result<String, EmitError> {
    let mut out = String::new();
    for (line_index, line) in partial.lines().iter().enumerate() {
        if line.tokens().is_empty() {
            return emit_error::EmptyLineSnafu.fail();
        }

        if line_index > 0 {
            out.push('\n');
        }
        for (token_index, token) in line.tokens().iter().enumerate() {
            if token_index > 0 {
                out.push(' ');
            }
            match token {
                PartialToken::Raw(raw) => {
                    validate_raw_token(raw)?;
                    out.push_str(raw);
                }
                PartialToken::Value(value) => {
                    if !value.belongs_to(handle) {
                        return emit_error::ForeignValueSnafu { value: value.id() }.fail();
                    }
                    let name = value_names
                        .get(&value.id())
                        .context(emit_error::UnknownValueSnafu { value: value.id() })?;
                    out.push_str(name);
                }
                PartialToken::Label(label) => {
                    if !label.belongs_to(handle) {
                        return emit_error::ForeignLabelSnafu { label: label.id() }.fail();
                    }
                    let line = labels
                        .get(label.id())
                        .context(emit_error::UnplacedLabelSnafu { label: label.id() })?;
                    out.push_str(&line.to_string());
                }
                PartialToken::Processor(_) => {
                    return emit_error::UnresolvedProcessorTokenSnafu.fail();
                }
            }
        }
    }
    Ok(out)
}

fn validate_raw_token(token: &str) -> Result<(), EmitError> {
    if token.is_empty() {
        return emit_error::EmptyRawTokenSnafu.fail();
    }
    if token.trim().is_empty() {
        return emit_error::BlankRawTokenSnafu.fail();
    }
    if token.chars().any(char::is_whitespace) {
        return emit_error::WhitespaceRawTokenSnafu {
            token: token.to_string(),
        }
        .fail();
    }

    Ok(())
}
