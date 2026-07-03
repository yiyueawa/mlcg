use mlcg_builtin_gen::{
    raw_statement::{scan_raw_statements, RawEnum},
    semantic_manifest::derive_semantic_manifest,
};

#[test]
fn derives_semantics_from_field_roles_and_enum_variants_without_statement_name_special_cases() {
    let source = r#"
        @RegisterStatement("copy")
        public static class CopyStatement extends LStatement{
            public String to = "result";
            public String from = "0";
            @Override public LInstruction build(LAssembler builder){ return new CopyI(builder.var(from), builder.var(to)); }
        }

        @RegisterStatement("calc")
        public static class CalcStatement extends LStatement{
            public LogicOp op = LogicOp.add;
            public String dest = "result", a = "a", b = "b";
            @Override public LInstruction build(LAssembler builder){ return new CalcI(op, builder.var(a), builder.var(b), builder.var(dest)); }
        }

        @RegisterStatement("branch")
        public static class BranchStatement extends LStatement{
            public int destIndex;
            public ConditionOp op = ConditionOp.notEqual;
            public String value = "x", compare = "false";
            @Override public LInstruction build(LAssembler builder){ return new BranchI(op, builder.var(value), builder.var(compare), destIndex); }
        }

        @RegisterStatement("read")
        public static class ReadStatement extends LStatement{
            public String output = "result", target = "cell1", address = "0";
            @Override public LInstruction build(LAssembler builder){ return new ReadI(builder.var(target), builder.var(address), builder.var(output)); }
        }
    "#;

    let mut raw = scan_raw_statements("fixture", source).expect("scan succeeds");
    raw.enums = vec![
        RawEnum {
            name: "LogicOp".to_string(),
            variants: vec!["add".to_string(), "not".to_string()],
            arities: [("not".to_string(), 1)].into_iter().collect(),
        },
        RawEnum {
            name: "ConditionOp".to_string(),
            variants: vec!["equal".to_string(), "always".to_string()],
            arities: std::collections::BTreeMap::new(),
        },
    ];

    let manifest = derive_semantic_manifest(&raw);

    let copy = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "copy")
        .expect("copy exists");
    assert_eq!(copy.emit, ["copy", "$to", "$from"]);
    assert_eq!(copy.receiver, "to");
    assert_eq!(copy.inputs, ["from"]);
    assert!(copy.outputs.is_empty());

    let calc_add = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "calc_add")
        .expect("calc_add exists");
    assert_eq!(calc_add.emit, ["calc", "add", "$dest", "$a", "$b"]);
    assert_eq!(calc_add.receiver, "a");
    assert_eq!(calc_add.inputs, ["b"]);
    assert_eq!(calc_add.outputs, ["dest"]);

    let calc_not = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "calc_not")
        .expect("calc_not exists");
    assert_eq!(calc_not.emit, ["calc", "not", "$dest", "$a", "0"]);
    assert_eq!(calc_not.receiver, "a");
    assert!(calc_not.inputs.is_empty());
    assert_eq!(calc_not.outputs, ["dest"]);

    let branch_equal = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "branch_equal")
        .expect("branch_equal exists");
    assert_eq!(
        branch_equal.emit,
        ["branch", "$destIndex", "equal", "$value", "$compare"]
    );
    assert_eq!(branch_equal.receiver, "value");
    assert_eq!(branch_equal.inputs, ["compare"]);
    assert_eq!(branch_equal.labels, ["destIndex"]);
    assert!(branch_equal.outputs.is_empty());

    let read = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "read")
        .expect("read exists");
    assert_eq!(read.receiver, "target");
    assert_eq!(read.inputs, ["address"]);
    assert_eq!(read.outputs, ["output"]);

    assert!(!manifest
        .instructions
        .iter()
        .any(|instruction| instruction.rust_name == "calc"));
    assert!(!manifest
        .instructions
        .iter()
        .any(|instruction| instruction.rust_name == "branch"));
}
