use std::{fmt, marker::PhantomData};

use crate::LabelId;

pub struct Label<P> {
    pub(crate) id: LabelId,
    pub(crate) _processor: PhantomData<P>,
}

impl<P> Clone for Label<P> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _processor: PhantomData,
        }
    }
}

impl<P> fmt::Debug for Label<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Label").field("id", &self.id).finish()
    }
}

impl<P> Label<P> {
    pub fn id(&self) -> LabelId {
        self.id
    }
}
