use mlcg_core::Processor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/basic_manifest.toml");
}

use generated::prelude::{ProcessorMultiExt, ProcessorOpAddExt};

struct P;

fn main() {
    let processor = Processor::<P>::new();

    processor.op_add_into(1, 2, 3);
    processor.multi_into((1, 2), 3);
}
