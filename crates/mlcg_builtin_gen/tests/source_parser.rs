use mlcg_builtin_gen::source_parser::parse_representative_manifest;

#[test]
fn parses_required_representative_entries() {
    let statements = r#"@RegisterStatement("set") public static class SetStatement {}"#;
    let logic_op = r#"
        public enum LogicOp{
            add("+", (a, b) -> a + b),
            not("flip", a -> ~(long)(a)),
        }
    "#;
    let condition_op = r#"
        public enum ConditionOp{
            equal("==", (a, b) -> true),
            always("always", (a, b) -> true);
        }
    "#;

    let manifest = parse_representative_manifest("158.1", statements, logic_op, condition_op)
        .expect("source parses");

    let names: Vec<_> = manifest
        .instructions
        .iter()
        .map(|instruction| instruction.rust_name.as_str())
        .collect();
    assert!(names.contains(&"set"));
    assert!(names.contains(&"op_add"));
    assert!(names.contains(&"op_not"));
    assert!(names.contains(&"jump_equal"));
    assert!(names.contains(&"jump_always"));
}
