use mlcg_core::{Processor, Value};

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/basic_manifest.toml");
}

use generated::prelude::{
    Arg, MultiOutput, MultiRecvOutput, ProcessorKeywordsExt, ProcessorMultiExt,
    ProcessorMultiRecvExt, ProcessorOpAddExt, ProcessorRecvOutExt, ProcessorSetExt,
    ValueMultiRecvExt, ValueRecvOutExt, ValueSetExt,
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
    processor.multi_into(multi.clone(), 2u64);
    processor.multi_into(&multi, 6u32);
    processor.multi_into(multi.as_tuple(), 8u32);
    let borrowed_multi_tuple: (&Value<P>, &Value<P>) = (&multi).into();
    processor.multi_into(borrowed_multi_tuple, 10u32);
    let owned_multi_tuple: (Value<P>, Value<P>) = multi.clone().into();
    processor.multi_into(owned_multi_tuple, 12u32);
    let (tuple_out_a, tuple_out_b) = processor.multi(typed_x.clone()).into_tuple();
    processor.multi_into((tuple_out_a, tuple_out_b), 4u32);
    let multi_recv: MultiRecvOutput<P> = x.multi_recv(y.clone());
    x.multi_recv_into(multi_recv.clone(), 3isize);
    x.multi_recv_into(&multi_recv, 7usize);
    x.multi_recv_into(multi_recv.as_tuple(), 9usize);
    let (recv_out_a, recv_out_b) = x.multi_recv(y.clone()).into_tuple();
    x.multi_recv_into((recv_out_a, recv_out_b), 5usize);
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
    assert!(text.contains("multi __mlcg_1 __mlcg_2 6"));
    assert!(text.contains("multi __mlcg_1 __mlcg_2 8"));
    assert!(text.contains("multi __mlcg_1 __mlcg_2 10"));
    assert!(text.contains("multi __mlcg_1 __mlcg_2 12"));
    assert!(text.contains("multi __mlcg_3 __mlcg_4 x"));
    assert!(text.contains("multi __mlcg_3 __mlcg_4 4"));
    assert!(text.contains("multi_recv __mlcg_5 __mlcg_6 x y"));
    assert!(text.contains("multi_recv __mlcg_5 __mlcg_6 x 3"));
    assert!(text.contains("multi_recv __mlcg_5 __mlcg_6 x 7"));
    assert!(text.contains("multi_recv __mlcg_5 __mlcg_6 x 9"));
    assert!(text.contains("multi_recv __mlcg_7 __mlcg_8 x y"));
    assert!(text.contains("multi_recv __mlcg_7 __mlcg_8 x 5"));
    assert!(text.contains("recv_out __mlcg_9 x y"));
    assert!(text.contains("recv_out __mlcg_9 x 1.5"));
    assert!(text.contains("keywords 4 5 6 __mlcg_10"));
    let _ = keyword_out;
}
