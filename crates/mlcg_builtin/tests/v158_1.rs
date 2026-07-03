use mlcg_builtin::latest::prelude::*;
use mlcg_core::Processor;

struct P;

#[test]
fn generated_v158_1_api_emits_representative_mlog() {
    let processor = Processor::<P>::new();
    let x = processor.named("x");
    let y = processor.named("y");

    x.set(1);
    let sum = processor.op_add(x.clone(), y.clone());
    let inverted = processor.op_not(sum.clone());
    processor.op_add_into(inverted.clone(), sum, 2);

    let output = processor.emit().expect("emit succeeds");

    assert_eq!(
        output,
        "set x 1\nop add __mlcg_0 x y\nop not __mlcg_1 __mlcg_0 0\nop add __mlcg_1 __mlcg_0 2"
    );
}
