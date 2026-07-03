#![forbid(unsafe_code)]

pub use mlcg_builtin as builtin;
pub use mlcg_core as core;

#[macro_export]
macro_rules! processor {
    () => {{
        struct MlcgProcessorMarker;
        $crate::core::Processor::<MlcgProcessorMarker>::new()
    }};
}

pub mod prelude {
    pub use crate::processor;
    pub use mlcg_builtin::latest::*;
    pub use mlcg_core::{
        Any, EmitError, Instruction, Label, LowerContext, PartialLine, PartialProgram,
        PartialToken, Processor, Value,
    };
}
