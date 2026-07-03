use std::fmt::Debug;

use crate::{
    label::Label,
    lower::{LowerContext, PartialProgram},
    LowerError,
};

pub trait Instruction<P>: Debug + Send + Sync + 'static {
    fn lower(
        &self,
        ctx: &mut LowerContext<P>,
        out: &mut PartialProgram<P>,
    ) -> Result<(), LowerError>;
}

#[derive(Debug)]
pub(crate) enum ProgramItem<P> {
    Instruction(Box<dyn Instruction<P>>),
    LabelPlacement(Label<P>),
}
