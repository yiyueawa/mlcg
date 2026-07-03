use std::collections::BTreeMap;

use snafu::OptionExt;

use crate::{
    error::emit_error,
    lower::{LabelTable, PartialProgram, PartialToken},
    EmitError, ValueId,
};

#[derive(Debug, Default)]
pub(crate) struct NameAllocator {
    names: BTreeMap<ValueId, String>,
    used: BTreeMap<String, usize>,
}

impl NameAllocator {
    pub(crate) fn name_for(&mut self, id: ValueId, hint: Option<&str>) -> String {
        if let Some(name) = self.names.get(&id) {
            return name.clone();
        }

        let base = hint.filter(|value| !value.is_empty()).unwrap_or("__mlcg");
        let count = self.used.entry(base.to_string()).or_insert(0);
        let name = if *count == 0 && base != "__mlcg" {
            base.to_string()
        } else {
            format!("{base}_{count}")
        };
        *count += 1;
        self.names.insert(id, name.clone());
        name
    }
}

pub(crate) fn emit_partial<P>(
    partial: &PartialProgram<P>,
    labels: &LabelTable,
    value_names: &BTreeMap<ValueId, String>,
) -> Result<String, EmitError> {
    let mut out = String::new();
    for (line_index, line) in partial.lines().iter().enumerate() {
        if line_index > 0 {
            out.push('\n');
        }
        for (token_index, token) in line.tokens().iter().enumerate() {
            if token_index > 0 {
                out.push(' ');
            }
            match token {
                PartialToken::Raw(raw) => out.push_str(raw),
                PartialToken::Value(value) => {
                    let name = value_names
                        .get(value)
                        .context(emit_error::UnknownValueSnafu { value: *value })?;
                    out.push_str(name);
                }
                PartialToken::Label(label) => {
                    let line = labels
                        .get(*label)
                        .context(emit_error::UnplacedLabelSnafu { label: *label })?;
                    out.push_str(&line.to_string());
                }
                PartialToken::Processor(_) => {}
            }
        }
    }
    Ok(out)
}
