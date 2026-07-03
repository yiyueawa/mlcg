use mlcg_core::Processor as CoreProcessor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/core_type_name_manifest.toml");
}

use generated::prelude::{ProcessorProcessorExt, ProcessorValueExt};

struct P;

fn main() {
    let processor = CoreProcessor::<P>::new();
    processor.value();
    processor.processor();

    let text = processor.emit().expect("emit succeeds");
    assert!(text.contains("value"));
    assert!(text.contains("processor"));
}
