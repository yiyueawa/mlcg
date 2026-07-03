use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
    sync::{Arc, Mutex},
};

use snafu::ResultExt;

use crate::{
    emit::{emit_partial, NameAllocator},
    error::emit_error,
    instruction::ProgramItem,
    label::Label,
    lower::{LabelTable, LowerContext, PartialProgram},
    value::{Any, Value},
    EmitError, Instruction, LabelId, ValueId,
};

static NEXT_VALUE_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_LABEL_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct ProgramState<P> {
    pub(crate) values: BTreeMap<ValueId, Option<String>>,
    pub(crate) items: Vec<ProgramItem<P>>,
    pub(crate) _processor: PhantomData<P>,
}

#[derive(Debug)]
pub struct ProcessorHandle<P> {
    pub(crate) state: Arc<Mutex<ProgramState<P>>>,
}

impl<P> Clone for ProcessorHandle<P> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
        }
    }
}

#[derive(Debug)]
pub struct Processor<P> {
    handle: ProcessorHandle<P>,
}

impl<P> Clone for Processor<P> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<P: 'static> Default for Processor<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: 'static> Processor<P> {
    pub fn new() -> Self {
        Self {
            handle: ProcessorHandle {
                state: Arc::new(Mutex::new(ProgramState {
                    values: BTreeMap::new(),
                    items: Vec::new(),
                    _processor: PhantomData,
                })),
            },
        }
    }

    pub fn handle(&self) -> ProcessorHandle<P> {
        self.handle.clone()
    }

    pub fn new_value(&self) -> Value<P, Any> {
        self.allocate_value(None)
    }

    pub fn named(&self, name: impl Into<String>) -> Value<P, Any> {
        self.allocate_value(Some(name.into()))
    }

    pub fn label(&self) -> Label<P> {
        let id = next_label_id();
        Label {
            id,
            _processor: PhantomData,
        }
    }

    pub fn place(&self, label: Label<P>) {
        self.handle.push_item(ProgramItem::LabelPlacement(label));
    }

    pub fn push<I>(&self, instruction: I)
    where
        I: Instruction<P>,
    {
        self.handle
            .push_item(ProgramItem::Instruction(Box::new(instruction)));
    }

    pub fn emit(&self) -> Result<String, EmitError> {
        self.handle.emit()
    }

    fn allocate_value(&self, name_hint: Option<String>) -> Value<P, Any> {
        let mut state = self
            .handle
            .state
            .lock()
            .expect("program state mutex poisoned");
        let id = next_value_id();
        state.values.insert(id, name_hint.clone());
        drop(state);
        Value {
            id,
            handle: self.handle.clone(),
            name_hint,
            _type: PhantomData,
        }
    }
}

impl<P: 'static> ProcessorHandle<P> {
    pub fn new_value(&self) -> Value<P, Any> {
        let mut state = self.state.lock().expect("program state mutex poisoned");
        let id = next_value_id();
        state.values.insert(id, None);
        drop(state);
        Value {
            id,
            handle: self.clone(),
            name_hint: None,
            _type: PhantomData,
        }
    }

    pub fn push<I>(&self, instruction: I)
    where
        I: Instruction<P>,
    {
        self.push_item(ProgramItem::Instruction(Box::new(instruction)));
    }
    pub(crate) fn push_item(&self, item: ProgramItem<P>) {
        let mut state = self.state.lock().expect("program state mutex poisoned");
        state.items.push(item);
    }

    pub(crate) fn emit(&self) -> Result<String, EmitError> {
        let state = self.state.lock().expect("program state mutex poisoned");
        let mut partial = PartialProgram::default();
        let mut labels = LabelTable::default();
        let mut lower_ctx = LowerContext::default();

        for item in &state.items {
            match item {
                ProgramItem::Instruction(instruction) => instruction
                    .lower(&mut lower_ctx, &mut partial)
                    .context(emit_error::LowerSnafu)?,
                ProgramItem::LabelPlacement(label) => {
                    labels.insert(label.id(), partial.line_count())?
                }
            }
        }

        let mut allocator = NameAllocator::default();
        let mut value_names = BTreeMap::new();
        for (id, hint) in &state.values {
            value_names.insert(*id, allocator.name_for(*id, hint.as_deref()));
        }

        emit_partial(&partial, &labels, &value_names)
    }
}

fn next_value_id() -> ValueId {
    ValueId(NEXT_VALUE_ID.fetch_add(1, Ordering::Relaxed))
}

fn next_label_id() -> LabelId {
    LabelId(NEXT_LABEL_ID.fetch_add(1, Ordering::Relaxed))
}
