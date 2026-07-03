use std::marker::PhantomData;

use crate::{processor::ProcessorHandle, ValueId};

#[derive(Debug, Clone, Copy, Default)]
pub struct Any;

#[derive(Debug)]
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
}
