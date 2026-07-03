#![forbid(unsafe_code)]

pub mod id;
pub mod processor;
pub mod value;

pub use id::{LabelId, ValueId};
pub use processor::{Processor, ProcessorHandle};
pub use value::{Any, Value};
