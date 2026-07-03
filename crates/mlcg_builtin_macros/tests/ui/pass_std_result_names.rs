use mlcg_core::Processor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/std_result_name_manifest.toml");
}

use generated::prelude::{ProcessorOkExt, ProcessorResultExt};

struct P;

fn main() {
    let processor = Processor::<P>::new();
    processor.result();
    processor.ok();

    let text = processor.emit().expect("emit succeeds");
    assert!(text.contains("result"));
    assert!(text.contains("ok"));
}
