use mlcg_core::Processor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/std_trait_name_manifest.toml");
}

use generated::prelude::ProcessorCloneExt;

struct P;

fn main() {
    let processor = Processor::<P>::new();
    let outputs = ProcessorCloneExt::clone(&processor);
    ProcessorCloneExt::clone_into(&processor, &outputs);

    let text = processor.emit().expect("emit succeeds");
    assert!(text.contains("clone __mlcg_0 __mlcg_1"));
}
