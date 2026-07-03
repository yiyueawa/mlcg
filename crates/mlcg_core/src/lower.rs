use std::{collections::BTreeMap, marker::PhantomData};

use snafu::ensure;

use crate::{error::emit_error, EmitError, Label, LabelId, Value, ValueId};

#[derive(Debug)]
pub struct LowerContext<P> {
    pub(crate) _processor: PhantomData<P>,
}

impl<P> Default for LowerContext<P> {
    fn default() -> Self {
        Self {
            _processor: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct PartialProgram<P> {
    lines: Vec<PartialLine<P>>,
}

impl<P> Default for PartialProgram<P> {
    fn default() -> Self {
        Self { lines: Vec::new() }
    }
}

impl<P> PartialProgram<P> {
    pub fn push_line(&mut self, line: PartialLine<P>) {
        self.lines.push(line);
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn lines(&self) -> &[PartialLine<P>] {
        &self.lines
    }
}

#[derive(Debug)]
pub struct PartialLine<P> {
    tokens: Vec<PartialToken<P>>,
}

impl<P> PartialLine<P> {
    pub fn new(tokens: Vec<PartialToken<P>>) -> Self {
        Self { tokens }
    }

    pub(crate) fn tokens(&self) -> &[PartialToken<P>] {
        &self.tokens
    }
}

#[derive(Debug)]
pub enum PartialToken<P> {
    Raw(String),
    Value(ValueId),
    Label(Label<P>),
    #[doc(hidden)]
    Processor(PhantomData<P>),
}

impl<P> PartialToken<P> {
    pub fn raw(token: impl Into<String>) -> Self {
        Self::Raw(token.into())
    }

    pub fn value<T>(value: Value<P, T>) -> Self {
        Self::Value(value.id())
    }

    pub fn label(label: Label<P>) -> Self {
        Self::Label(label)
    }
}

#[derive(Debug, Default)]
pub(crate) struct LabelTable {
    lines: BTreeMap<LabelId, usize>,
}

impl LabelTable {
    pub(crate) fn insert(&mut self, label: LabelId, line: usize) -> Result<(), EmitError> {
        ensure!(
            !self.lines.contains_key(&label),
            emit_error::DuplicateLabelPlacementSnafu { label }
        );
        self.lines.insert(label, line);
        Ok(())
    }

    pub(crate) fn get(&self, label: LabelId) -> Option<usize> {
        self.lines.get(&label).copied()
    }
}
