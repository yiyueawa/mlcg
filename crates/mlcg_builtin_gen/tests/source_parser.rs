use std::{fs, path::Path};

use mlcg_builtin_gen::source_parser::{parse_cached_mindustry, scan_cached_mindustry_raw};

#[test]
fn cached_mindustry_parser_derives_full_semantic_manifest() {
    let cache = tempfile::tempdir().expect("tempdir");
    let logic_dir = cache.path().join("core/src/mindustry/logic");
    fs::create_dir_all(&logic_dir).expect("create logic dir");

    write(
        &logic_dir.join("LStatements.java"),
        r#"
            public class LStatements{
                @RegisterStatement("set")
                public static class SetStatement extends LStatement{
                    public String to = "result", from = "0";
                    @Override public LInstruction build(LAssembler builder){ return new SetI(builder.var(from), builder.var(to)); }
                }

                @RegisterStatement("select")
                public static class SelectStatement extends LStatement{
                    public String result = "result";
                    public ConditionOp op = ConditionOp.equal;
                    public String comp0 = "x", comp1 = "false", a = "a", b = "b";
                    @Override public LInstruction build(LAssembler builder){ return new SelectI(op, builder.var(result), builder.var(comp0), builder.var(comp1), builder.var(a), builder.var(b)); }
                }
            }
        "#,
    );
    write(
        &logic_dir.join("LogicOp.java"),
        r#"
            public enum LogicOp{
                add("+", (a, b) -> a + b),
                not("flip", a -> ~(long)(a));
            }
        "#,
    );
    write(
        &logic_dir.join("ConditionOp.java"),
        r#"
            public enum ConditionOp{
                equal("==", (a, b) -> Math.abs(a - b) < 0.000001),
                always("always", (a, b) -> true);
            }
        "#,
    );

    let manifest = parse_cached_mindustry("fixture", cache.path()).expect("source parses");
    let names: Vec<_> = manifest
        .instructions
        .iter()
        .map(|instruction| instruction.rust_name.as_str())
        .collect();

    assert!(names.contains(&"set"));
    assert!(names.contains(&"select_equal"));
    assert!(names.contains(&"select_always"));
    assert!(!names.contains(&"op_add"));

    let select_always = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "select_always")
        .expect("select_always exists");
    assert_eq!(
        select_always.emit,
        ["select", "$result", "always", "0", "0", "$a", "$b"]
    );
    assert_eq!(select_always.inputs, ["a", "b"]);
}

#[test]
fn cached_mindustry_parser_reports_missing_enum_sources() {
    let cache = tempfile::tempdir().expect("tempdir");
    let logic_dir = cache.path().join("core/src/mindustry/logic");
    fs::create_dir_all(&logic_dir).expect("create logic dir");

    write(
        &logic_dir.join("LStatements.java"),
        r#"
            public class LStatements{
                @RegisterStatement("calc")
                public static class CalcStatement extends LStatement{
                    public MissingOp op = MissingOp.add;
                    public String dest = "result", a = "a", b = "b";
                    @Override public LInstruction build(LAssembler builder){ return new CalcI(op, builder.var(a), builder.var(b), builder.var(dest)); }
                }
            }
        "#,
    );

    let error = scan_cached_mindustry_raw("fixture", cache.path())
        .expect_err("missing enum source should fail");

    assert_eq!(
        error.to_string(),
        "required source item not found: enum MissingOp"
    );
}

fn write(path: &Path, content: &str) {
    fs::write(path, content).expect("write source file");
}
