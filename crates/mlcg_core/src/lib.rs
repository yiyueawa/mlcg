#![forbid(unsafe_code)]

mod emit;
pub mod error;
pub mod id;
pub mod instruction;
pub mod label;
pub mod lower;
pub mod processor;
pub mod value;

pub use error::{EmitError, LowerError};
pub use id::{LabelId, ValueId};
pub use instruction::Instruction;
pub use label::Label;
pub use lower::{LowerContext, PartialLine, PartialProgram, PartialToken};
pub use processor::{Processor, ProcessorHandle};
pub use value::{Any, Value};
