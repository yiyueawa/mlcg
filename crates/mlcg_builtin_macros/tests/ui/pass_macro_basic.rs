use mlcg_core::{Processor, Value};

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/basic_manifest.toml");
}

use generated::prelude::{
    Arg, MultiOutput, MultiRecvOutput, ProcessorKeywordsExt, ProcessorMultiExt, ProcessorMultiRecvExt,
    ProcessorOpAddExt, ProcessorRecvOutExt, ProcessorSetExt, ValueMultiRecvExt, ValueRecvOutExt,
    ValueSetExt,
};

struct P;
struct Marker;

fn assert_imported()
where
    Arg<P>: Sized,
    Processor<P>: ProcessorSetExt<P>
        + ProcessorOpAddExt<P>
        + ProcessorMultiExt<P>
        + ProcessorMultiRecvExt<P>
        + ProcessorRecvOutExt<P>
        + ProcessorKeywordsExt<P>,
    Value<P>: ValueSetExt<P> + ValueMultiRecvExt<P> + ValueRecvOutExt<P>,
    Value<P, Marker>: ValueSetExt<P> + ValueMultiRecvExt<P> + ValueRecvOutExt<P>,
{
}

fn main() {
    assert_imported();

    let processor = Processor::<P>::new();
    let x = processor.named("x");
    let y = processor.named("y");
    let typed_x = x.cast::<Marker>();

    x.set(1u32);
    typed_x.set(8);
    y.set(true);
    let out = processor.op_add(typed_x.clone(), y.clone());
    processor.op_add_into(out, typed_x.clone(), y.clone());
    let multi: MultiOutput<P> = processor.multi(typed_x.clone());
    processor.multi_into(multi.outA.clone(), multi.outB.clone(), 2u64);
    let multi_recv: MultiRecvOutput<P> = x.multi_recv(y.clone());
    x.multi_recv_into(multi_recv.outA.clone(), multi_recv.outB.clone(), 3isize);
    let recv_out = typed_x.recv_out(y.clone());
    typed_x.recv_out_into(recv_out.clone(), 1.5f32);
    let keyword_out = processor.keywords(4, 5, 6);

    let text = processor.emit().unwrap();
    assert!(text.contains("set x 1"));
    assert!(text.contains("set x 8"));
    assert!(text.contains("set y true"));
    assert!(text.contains("op add"));
    assert!(text.contains("multi __mlcg_1 __mlcg_2 x"));
    assert!(text.contains("multi __mlcg_1 __mlcg_2 2"));
    assert!(text.contains("multi_recv __mlcg_3 __mlcg_4 x y"));
    assert!(text.contains("multi_recv __mlcg_3 __mlcg_4 x 3"));
    assert!(text.contains("recv_out __mlcg_5 x y"));
    assert!(text.contains("recv_out __mlcg_5 x 1.5"));
    assert!(text.contains("keywords 4 5 6 __mlcg_6"));
    let _ = keyword_out;
}
