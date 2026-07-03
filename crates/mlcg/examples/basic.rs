use mlcg::prelude::*;

fn main() {
    let processor = processor!();

    let x = processor.named("x");
    let y = processor.named("y");

    x.set(1);
    let sum = processor.op_add(x.clone(), y);
    let inverted = processor.op_not(sum);
    inverted.set(0);

    println!("{}", processor.emit().expect("program emits"));
}
