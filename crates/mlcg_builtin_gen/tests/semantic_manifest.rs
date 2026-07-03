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

        @RegisterStatement("select")
        public static class SelectStatement extends LStatement{
            public String result = "result";
            public ConditionOp op = ConditionOp.equal;
            public String comp0 = "x", comp1 = "y", a = "then", b = "else";
            @Override public LInstruction build(LAssembler builder){ return new SelectI(op, builder.var(result), builder.var(comp0), builder.var(comp1), builder.var(a), builder.var(b)); }
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
            arities: [("equal".to_string(), 2), ("always".to_string(), 0)]
                .into_iter()
                .collect(),
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

    let branch_always = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "branch_always")
        .expect("branch_always exists");
    assert_eq!(
        branch_always.emit,
        ["branch", "$destIndex", "always", "0", "0"]
    );
    assert!(branch_always.receiver.is_empty());
    assert!(branch_always.inputs.is_empty());
    assert_eq!(branch_always.labels, ["destIndex"]);
    assert!(branch_always.outputs.is_empty());

    let select_equal = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "select_equal")
        .expect("select_equal exists");
    assert_eq!(
        select_equal.emit,
        ["select", "$result", "equal", "$comp0", "$comp1", "$a", "$b"]
    );
    assert_eq!(select_equal.receiver, "comp0");
    assert_eq!(select_equal.inputs, ["comp1", "a", "b"]);
    assert_eq!(select_equal.outputs, ["result"]);

    let select_always = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "select_always")
        .expect("select_always exists");
    assert_eq!(
        select_always.emit,
        ["select", "$result", "always", "0", "0", "$a", "$b"]
    );
    assert!(select_always.receiver.is_empty());
    assert_eq!(select_always.inputs, ["a", "b"]);
    assert_eq!(select_always.outputs, ["result"]);

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
fn normalizes_generated_rust_names_from_source_symbols() {
    let source = r#"
        @RegisterStatement("odd-op")
        public static class OddOpStatement extends LStatement{
            public StrangeOp op = StrangeOp.two__words;
            public String result = "result", lhs = "a", rhs = "b";
            @Override public LInstruction build(LAssembler builder){ return new OddOpI(op, builder.var(result), builder.var(lhs), builder.var(rhs)); }
        }
    "#;

    let mut raw = scan_raw_statements("fixture", source).expect("scan succeeds");
    raw.enums = vec![RawEnum {
        name: "StrangeOp".to_string(),
        variants: vec!["two--words".to_string()],
        arities: [("two--words".to_string(), 2)].into_iter().collect(),
    }];

    let manifest = derive_semantic_manifest(&raw);

    assert!(
        manifest
            .instructions
            .iter()
            .any(|instruction| instruction.rust_name == "odd_op_two_words"),
        "generated rust name should collapse separators"
    );

    let source = r#"
        @RegisterStatement("2-op")
        public static class NumberedOpStatement extends LStatement{
            public StrangeOp op = StrangeOp.three;
            public String result = "result", lhs = "a", rhs = "b";
            @Override public LInstruction build(LAssembler builder){ return new NumberedOpI(op, builder.var(result), builder.var(lhs), builder.var(rhs)); }
        }
    "#;

    let mut raw = scan_raw_statements("fixture", source).expect("scan succeeds");
    raw.enums = vec![RawEnum {
        name: "StrangeOp".to_string(),
        variants: vec!["3-way".to_string()],
        arities: [("3-way".to_string(), 2)].into_iter().collect(),
    }];

    let manifest = derive_semantic_manifest(&raw);

    assert!(
        manifest
            .instructions
            .iter()
            .any(|instruction| instruction.rust_name == "symbol_2_op_symbol_3_way"),
        "generated rust name should not start any segment with a digit"
    );
}

#[test]
fn expands_mode_selector_fields_when_enum_metadata_exists() {
    let source = r#"
        @RegisterStatement("basic_selector")
        public static class BasicSelectorStatement extends LStatement{
            public Mode mode = Mode.sum;
            public String result = "result", lhs = "a", rhs = "b";
            @Override public LInstruction build(LAssembler builder){ return new BasicSelectorI(mode, builder.var(result), builder.var(lhs), builder.var(rhs)); }
        }
    "#;

    let mut raw = scan_raw_statements("fixture", source).expect("scan succeeds");
    raw.enums = vec![RawEnum {
        name: "Mode".to_string(),
        variants: vec!["sum".to_string(), "identity".to_string()],
        arities: [("sum".to_string(), 2), ("identity".to_string(), 1)]
            .into_iter()
            .collect(),
    }];

    let manifest = derive_semantic_manifest(&raw);

    let sum = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "basic_selector_sum")
        .expect("sum variant exists");
    assert_eq!(
        sum.emit,
        ["basic_selector", "sum", "$result", "$lhs", "$rhs"]
    );
    assert_eq!(sum.receiver, "lhs");
    assert_eq!(sum.inputs, ["rhs"]);
    assert_eq!(sum.outputs, ["result"]);

    let identity = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "basic_selector_identity")
        .expect("identity variant exists");
    assert_eq!(
        identity.emit,
        ["basic_selector", "identity", "$result", "$lhs", "0"]
    );
    assert_eq!(identity.receiver, "lhs");
    assert!(identity.inputs.is_empty());
    assert_eq!(identity.outputs, ["result"]);

    assert!(!manifest
        .instructions
        .iter()
        .any(|instruction| instruction.rust_name == "basic_selector"));
}

#[test]
fn avoids_fallback_receiver_for_plain_output_queries_without_preferred_subject() {
    let source = r#"
        @RegisterStatement("link_like")
        public static class LinkLikeStatement extends LStatement{
            public String output = "result", address = "0";
            @Override public LInstruction build(LAssembler builder){ return new LinkLikeI(builder.var(output), builder.var(address)); }
        }

        @RegisterStatement("color_like")
        public static class ColorLikeStatement extends LStatement{
            public String result = "result", r = "1", g = "1", b = "1", a = "1";
            @Override public LInstruction build(LAssembler builder){ return new ColorLikeI(builder.var(result), builder.var(r), builder.var(g), builder.var(b), builder.var(a)); }
        }
    "#;

    let raw = scan_raw_statements("fixture", source).expect("scan succeeds");
    let manifest = derive_semantic_manifest(&raw);

    let link_like = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "link_like")
        .expect("link_like exists");
    assert_eq!(link_like.emit, ["link_like", "$output", "$address"]);
    assert!(link_like.receiver.is_empty());
    assert_eq!(link_like.inputs, ["address"]);
    assert_eq!(link_like.outputs, ["output"]);

    let color_like = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "color_like")
        .expect("color_like exists");
    assert_eq!(
        color_like.emit,
        ["color_like", "$result", "$r", "$g", "$b", "$a"]
    );
    assert!(color_like.receiver.is_empty());
    assert_eq!(color_like.inputs, ["r", "g", "b", "a"]);
    assert_eq!(color_like.outputs, ["result"]);
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

#[test]
fn emits_ignored_serialized_fields_as_defaults_without_api_parameters() {
    let source = r#"
        @RegisterStatement("radar_like")
        public static class RadarLikeStatement extends LStatement{
            public RadarTarget target1 = RadarTarget.enemy;
            public String radar = "turret1", sortOrder = "1", output = "result";
            public RadarLikeStatement(){
                radar = "0";
            }
            @Override public LInstruction build(LAssembler builder){
                return new RadarLikeI(target1, builder.var("@unit"), builder.var(sortOrder), builder.var(output));
            }
        }
    "#;

    let mut raw = scan_raw_statements("fixture", source).expect("scan succeeds");
    raw.enums = vec![RawEnum {
        name: "RadarTarget".to_string(),
        variants: vec!["enemy".to_string()],
        arities: std::collections::BTreeMap::new(),
    }];
    let manifest = derive_semantic_manifest(&raw);
    let radar = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "radar_like")
        .expect("radar_like exists");

    assert_eq!(
        radar.emit,
        ["radar_like", "$target1", "0", "$sortOrder", "$output"]
    );
    assert!(radar.receiver.is_empty());
    assert_eq!(radar.inputs, ["target1", "sortOrder"]);
    assert_eq!(radar.outputs, ["output"]);
}
