use mlcg_core::Processor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/keyword_item_name_manifest.toml");
}

use generated::prelude::ProcessorArgSelfExt;

struct P;

fn main() {
    let processor = Processor::<P>::new();
    processor.arg_self();

    let text = processor.emit().expect("emit succeeds");
    assert!(text.contains("self"));
}
