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

use mlcg_core::{Instruction, Label, LowerContext, PartialLine, PartialProgram, PartialToken};

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
