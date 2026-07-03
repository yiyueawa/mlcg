use mlcg_core::Processor;

struct TestProcessor;
struct Marker;

#[test]
fn processor_creates_named_and_temporary_values() {
    let processor = Processor::<TestProcessor>::new();

    let named = processor.named("x");
    let temporary = processor.new_value();

    assert_eq!(named.name_hint().as_deref(), Some("x"));
    assert_eq!(temporary.name_hint(), None);
    assert_ne!(named.id(), temporary.id());
}

#[test]
fn values_can_be_retagged_without_changing_identity() {
    let processor = Processor::<TestProcessor>::new();

    let named = processor.named("x");
    let typed = named.cast::<Marker>();
    let erased = typed.erase_type();

    assert_eq!(typed.id(), named.id());
    assert_eq!(erased.id(), named.id());
    assert_eq!(erased.name_hint().as_deref(), Some("x"));
}

use mlcg_core::{
    Instruction, Label, LowerContext, PartialLine, PartialProgram, PartialToken, Value,
};

#[derive(Debug)]
struct RawLine(&'static [&'static str]);

impl Instruction<TestProcessor> for RawLine {
    fn lower(
        &self,
        _ctx: &mut LowerContext<TestProcessor>,
        out: &mut PartialProgram<TestProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(
            self.0.iter().copied().map(PartialToken::raw).collect(),
        ));
        Ok(())
    }
}

#[derive(Debug)]
struct JumpTo(Label<TestProcessor>);

impl Instruction<TestProcessor> for JumpTo {
    fn lower(
        &self,
        _ctx: &mut LowerContext<TestProcessor>,
        out: &mut PartialProgram<TestProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(vec![
            PartialToken::raw("jump"),
            PartialToken::label(self.0.clone()),
            PartialToken::raw("always"),
            PartialToken::raw("0"),
            PartialToken::raw("0"),
        ]));
        Ok(())
    }
}

#[derive(Debug)]
struct TwoLines;

impl Instruction<TestProcessor> for TwoLines {
    fn lower(
        &self,
        _ctx: &mut LowerContext<TestProcessor>,
        out: &mut PartialProgram<TestProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(vec![PartialToken::raw("noop")]));
        out.push_line(PartialLine::new(vec![PartialToken::raw("end")]));
        Ok(())
    }
}

#[derive(Debug)]
struct PrintValue(Value<TestProcessor>);

impl Instruction<TestProcessor> for PrintValue {
    fn lower(
        &self,
        _ctx: &mut LowerContext<TestProcessor>,
        out: &mut PartialProgram<TestProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(vec![
            PartialToken::raw("print"),
            PartialToken::value(self.0.clone()),
        ]));
        Ok(())
    }
}

#[test]
fn labels_resolve_after_multiline_lowering() {
    let processor = Processor::<TestProcessor>::new();
    let target = processor.label();

    processor.push(RawLine(&["set", "x", "1"]));
    processor.push(TwoLines);
    processor.push(JumpTo(target.clone()));
    processor.place(target);
    processor.push(RawLine(&["set", "x", "2"]));

    let output = processor.emit().expect("emit succeeds");

    assert_eq!(output, "set x 1\nnoop\nend\njump 4 always 0 0\nset x 2");
}

#[test]
fn unplaced_label_is_an_emit_error() {
    let processor = Processor::<TestProcessor>::new();
    let missing = processor.label();
    processor.push(JumpTo(missing));

    let error = processor.emit().expect_err("label is not placed");

    assert!(error.to_string().contains("unplaced label"));
}

#[test]
fn duplicate_label_placement_is_an_emit_error() {
    let processor = Processor::<TestProcessor>::new();
    let label = processor.label();

    processor.place(label.clone());
    processor.push(RawLine(&["noop"]));
    processor.place(label);

    let error = processor.emit().expect_err("label is placed twice");

    assert!(error.to_string().contains("duplicate label placement"));
}

#[test]
fn foreign_processor_value_does_not_alias_local_value_with_same_type() {
    let local_processor = Processor::<TestProcessor>::new();
    let foreign_processor = Processor::<TestProcessor>::new();

    let _local = local_processor.named("local");
    let foreign = foreign_processor.named("foreign");
    local_processor.push(PrintValue(foreign));

    let error = local_processor
        .emit()
        .expect_err("foreign value is not part of local processor state");

    assert!(error.to_string().contains("unknown value"));
}

#[test]
fn temporary_value_names_do_not_collide_with_explicit_names() {
    let processor = Processor::<TestProcessor>::new();

    let explicit = processor.named("__mlcg_0");
    let temporary = processor.new_value();
    processor.push(PrintValue(explicit));
    processor.push(PrintValue(temporary));

    let output = processor.emit().expect("emit succeeds");

    assert_eq!(output, "print __mlcg_0\nprint __mlcg_1");
}

#[test]
fn foreign_processor_label_does_not_alias_local_label_with_same_type() {
    let local_processor = Processor::<TestProcessor>::new();
    let foreign_processor = Processor::<TestProcessor>::new();

    let local = local_processor.label();
    let foreign = foreign_processor.label();
    local_processor.push(JumpTo(foreign));
    local_processor.push(RawLine(&["noop"]));
    local_processor.place(local);

    let error = local_processor
        .emit()
        .expect_err("foreign label is not part of local processor state");

    assert!(error.to_string().contains("unplaced label"));
}

#[test]
fn foreign_processor_label_placement_is_an_emit_error() {
    let local_processor = Processor::<TestProcessor>::new();
    let foreign_processor = Processor::<TestProcessor>::new();

    let foreign = foreign_processor.label();
    local_processor.push(JumpTo(foreign.clone()));
    local_processor.place(foreign);

    let error = local_processor
        .emit()
        .expect_err("foreign label placement is not part of local processor state");

    assert!(error.to_string().contains("foreign label"));
}
