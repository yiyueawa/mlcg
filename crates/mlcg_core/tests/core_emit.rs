use mlcg_core::Processor;

struct TestProcessor;

#[test]
fn processor_creates_named_and_temporary_values() {
    let processor = Processor::<TestProcessor>::new();

    let named = processor.named("x");
    let temporary = processor.new_value();

    assert_eq!(named.name_hint().as_deref(), Some("x"));
    assert_eq!(temporary.name_hint(), None);
    assert_ne!(named.id(), temporary.id());
}
