use mlcg_core::Processor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/basic_manifest.toml");
}

use generated::prelude::*;

struct P;

fn main() {
    let processor = Processor::<P>::new();
    let x = processor.named("x");
    let y = processor.named("y");

    x.set(1);
    let out = processor.op_add(x.clone(), y.clone());
    processor.op_add_into(out, x, y);

    let text = processor.emit().unwrap();
    assert!(text.contains("set x 1"));
    assert!(text.contains("op add"));
}
