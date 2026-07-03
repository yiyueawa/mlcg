use mlcg::prelude::*;

fn accepts_same<P>(_: Value<P>, _: Value<P>) {}

fn main() {
    let first = processor!();
    let second = processor!();
    accepts_same(first.new_value(), second.new_value());
}
