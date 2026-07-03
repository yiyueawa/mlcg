use mlcg::prelude::*;

fn accepts_same<P>(_: Label<P>, _: Label<P>) {}

fn main() {
    let first = processor!();
    let second = processor!();
    accepts_same(first.label(), second.label());
}
