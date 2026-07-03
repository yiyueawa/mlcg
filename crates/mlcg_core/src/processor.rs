use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::{
    value::{Any, Value},
    ValueId,
};

#[derive(Debug)]
pub(crate) struct ProgramState<P> {
    pub(crate) next_value: u64,
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

#[derive(Debug, Clone)]
pub struct Processor<P> {
    handle: ProcessorHandle<P>,
}

impl<P> Default for Processor<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> Processor<P> {
    pub fn new() -> Self {
        Self {
            handle: ProcessorHandle {
                state: Arc::new(Mutex::new(ProgramState {
                    next_value: 0,
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

    fn allocate_value(&self, name_hint: Option<String>) -> Value<P, Any> {
        let mut state = self
            .handle
            .state
            .lock()
            .expect("program state mutex poisoned");
        let id = ValueId(state.next_value);
        state.next_value += 1;
        drop(state);
        Value {
            id,
            handle: self.handle.clone(),
            name_hint,
            _type: PhantomData,
        }
    }
}
