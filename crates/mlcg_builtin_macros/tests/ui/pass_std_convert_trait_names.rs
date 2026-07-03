use mlcg_core::Processor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/std_convert_trait_name_manifest.toml");
}

use generated::prelude::{ProcessorFromExt, ProcessorIntoExt};

struct P;

fn main() {
    let processor = Processor::<P>::new();
    let from_output = ProcessorFromExt::from(&processor, 1);
    ProcessorFromExt::from_into(&processor, &from_output, 2);
    let into_output = ProcessorIntoExt::into(&processor, 3);
    ProcessorIntoExt::into_into(&processor, &into_output, 4);

    let text = processor.emit().expect("emit succeeds");
    assert!(text.contains("from __mlcg_0 1"));
    assert!(text.contains("from __mlcg_0 2"));
    assert!(text.contains("into __mlcg_1 3"));
    assert!(text.contains("into __mlcg_1 4"));
}
