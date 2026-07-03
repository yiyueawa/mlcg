use mlcg_core::{Processor, Value};

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/basic_manifest.toml");
}

use generated::prelude::{
    Arg, MultiOutput, MultiRecvOutput, ProcessorMultiExt, ProcessorMultiRecvExt,
    ProcessorOpAddExt, ProcessorSetExt, ValueMultiRecvExt, ValueSetExt,
};

struct P;

fn assert_imported()
where
    Arg<P>: Sized,
    Processor<P>: ProcessorSetExt<P> + ProcessorOpAddExt<P> + ProcessorMultiExt<P> + ProcessorMultiRecvExt<P>,
    Value<P>: ValueSetExt<P> + ValueMultiRecvExt<P>,
{
}

fn main() {
    assert_imported();

    let processor = Processor::<P>::new();
    let x = processor.named("x");
    let y = processor.named("y");

    x.set(1);
    let out = processor.op_add(x.clone(), y.clone());
    processor.op_add_into(out, x.clone(), y.clone());
    let multi: MultiOutput<P> = processor.multi(x.clone());
    processor.multi_into(multi.outA.clone(), multi.outB.clone(), y.clone());
    let multi_recv: MultiRecvOutput<P> = x.multi_recv(y.clone());
    x.multi_recv_into(multi_recv.outA.clone(), multi_recv.outB.clone(), 2);

    let text = processor.emit().unwrap();
    assert!(text.contains("set x 1"));
    assert!(text.contains("op add"));
    assert!(text.contains("multi __mlcg_1 __mlcg_2 x"));
    assert!(text.contains("multi __mlcg_1 __mlcg_2 y"));
    assert!(text.contains("multi_recv __mlcg_3 __mlcg_4 x y"));
    assert!(text.contains("multi_recv __mlcg_3 __mlcg_4 x 2"));
}
