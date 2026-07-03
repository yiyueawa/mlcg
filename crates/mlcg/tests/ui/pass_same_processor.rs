use mlcg::prelude::*;

fn accepts_same<P>(_: Value<P>, _: Value<P>) {}

fn main() {
    let processor = processor!();
    let a = processor.new_value();
    let b = processor.named("b");
    accepts_same(a, b);
}
