use std::{fmt, marker::PhantomData};

use crate::{processor::ProcessorHandle, ValueId};

#[derive(Debug, Clone, Copy, Default)]
pub struct Any;

pub struct Value<P, T = Any> {
    pub(crate) id: ValueId,
    pub(crate) handle: ProcessorHandle<P>,
    pub(crate) name_hint: Option<String>,
    pub(crate) _type: PhantomData<T>,
}

impl<P, T> Clone for Value<P, T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            handle: self.handle.clone(),
            name_hint: self.name_hint.clone(),
            _type: PhantomData,
        }
    }
}

impl<P, T> fmt::Debug for Value<P, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Value")
            .field("id", &self.id)
            .field("name_hint", &self.name_hint)
            .finish()
    }
}

impl<P, T> Value<P, T> {
    pub fn id(&self) -> ValueId {
        self.id
    }

    pub fn name_hint(&self) -> Option<String> {
        self.name_hint.clone()
    }

    pub fn handle(&self) -> ProcessorHandle<P> {
        self.handle.clone()
    }

    pub fn cast<U>(&self) -> Value<P, U> {
        Value {
            id: self.id,
            handle: self.handle.clone(),
            name_hint: self.name_hint.clone(),
            _type: PhantomData,
        }
    }

    pub fn erase_type(&self) -> Value<P, Any> {
        self.cast()
    }
}

impl<P: 'static, T> Value<P, T> {
    pub(crate) fn belongs_to(&self, handle: &ProcessorHandle<P>) -> bool {
        self.handle.same_state(handle)
    }
}
