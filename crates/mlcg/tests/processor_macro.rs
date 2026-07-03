#[test]
fn processor_macro_creates_processors() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_same_processor.rs");
    t.compile_fail("tests/ui/mixed_processors.rs");
    t.compile_fail("tests/ui/mixed_processor_labels.rs");
}
