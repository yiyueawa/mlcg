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

#[test]
fn derives_receiver_and_output_roles_from_field_names() {
    let source = r#"
        @RegisterStatement("write_like")
        public static class WriteLikeStatement extends LStatement{
            public String input = "result", target = "cell1", address = "0";
            @Override public LInstruction build(LAssembler builder){ return new WriteLikeI(builder.var(target), builder.var(address), builder.var(input)); }
        }

        @RegisterStatement("sense_like")
        public static class SenseLikeStatement extends LStatement{
            public String to = "result", from = "block1", type = "@enabled";
            @Override public LInstruction build(LAssembler builder){ return new SenseLikeI(builder.var(from), builder.var(to), builder.var(type)); }
        }
    "#;

    let raw = scan_raw_statements("fixture", source).expect("scan succeeds");
    let manifest = derive_semantic_manifest(&raw);

    let write_like = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "write_like")
        .expect("write_like exists");
    assert_eq!(
        write_like.emit,
        ["write_like", "$input", "$target", "$address"]
    );
    assert_eq!(write_like.receiver, "target");
    assert_eq!(write_like.inputs, ["input", "address"]);
    assert!(write_like.outputs.is_empty());

    let sense_like = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "sense_like")
        .expect("sense_like exists");
    assert_eq!(sense_like.emit, ["sense_like", "$to", "$from", "$type"]);
    assert_eq!(sense_like.receiver, "from");
    assert_eq!(sense_like.inputs, ["type"]);
    assert_eq!(sense_like.outputs, ["to"]);
}

#[test]
fn keeps_multiple_outputs_and_avoids_fallback_receiver_for_multi_output_queries() {
    let source = r#"
        @RegisterStatement("locate_like")
        public static class LocateLikeStatement extends LStatement{
            public LLocate locate = LLocate.building;
            public BlockFlag flag = BlockFlag.core;
            public String enemy = "true", ore = "@copper", outX = "outx", outY = "outy", outFound = "found", outBuild = "building";
            @Override public LInstruction build(LAssembler builder){ return new LocateLikeI(locate, flag, builder.var(enemy), builder.var(ore), builder.var(outX), builder.var(outY), builder.var(outFound), builder.var(outBuild)); }
        }
    "#;

    let mut raw = scan_raw_statements("fixture", source).expect("scan succeeds");
    raw.enums = vec![RawEnum {
        name: "LLocate".to_string(),
        variants: vec!["building".to_string()],
        arities: std::collections::BTreeMap::new(),
    }];

    let manifest = derive_semantic_manifest(&raw);
    let locate = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "locate_like_building")
        .expect("locate_like_building exists");

    assert_eq!(
        locate.emit,
        [
            "locate_like",
            "building",
            "$flag",
            "$enemy",
            "$ore",
            "$outX",
            "$outY",
            "$outFound",
            "$outBuild",
        ]
    );
    assert!(locate.receiver.is_empty());
    assert_eq!(locate.inputs, ["flag", "enemy", "ore"]);
    assert_eq!(locate.outputs, ["outX", "outY", "outFound", "outBuild"]);
}
