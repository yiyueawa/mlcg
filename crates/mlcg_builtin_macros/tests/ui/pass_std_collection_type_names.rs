use mlcg_core::Processor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/std_collection_type_name_manifest.toml");
}

use generated::prelude::{ProcessorStringExt, ProcessorVecExt};

struct P;

fn main() {
    let processor = Processor::<P>::new();
    processor.string("text");
    processor.vec(1);

    let text = processor.emit().expect("emit succeeds");
    assert!(text.contains("string text"));
    assert!(text.contains("vec 1"));
}
