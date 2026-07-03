use mlcg_builtin_gen::{
    raw_statement::scan_raw_statements, semantic_manifest::derive_semantic_manifest,
};

#[test]
fn derives_known_semantic_overrides_and_generic_statement_apis() {
    let source = r#"
        @RegisterStatement("read")
        public static class ReadStatement extends LStatement{
            public String output = "result", target = "cell1", address = "0";
            @Override public LInstruction build(LAssembler builder){
                return new ReadI(builder.var(target), builder.var(address), builder.var(output));
            }
        }

        @RegisterStatement("set")
        public static class SetStatement extends LStatement{
            public String to = "result";
            public String from = "0";
            @Override public LInstruction build(LAssembler builder){ return new SetI(builder.var(from), builder.var(to)); }
        }

        @RegisterStatement("print")
        public static class PrintStatement extends LStatement{
            public String value = "message";
            @Override public LInstruction build(LAssembler builder){ return new PrintI(builder.var(value)); }
        }

        @RegisterStatement("op")
        public static class OperationStatement extends LStatement{
            public LogicOp op = LogicOp.add;
            public String dest = "result", a = "a", b = "b";
            @Override public LInstruction build(LAssembler builder){ return new OpI(op, builder.var(a), builder.var(b), builder.var(dest)); }
        }

        @RegisterStatement("jump")
        public static class JumpStatement extends LStatement{
            public int destIndex;
            public ConditionOp op = ConditionOp.notEqual;
            public String value = "x", compare = "false";
            @Override public LInstruction build(LAssembler builder){ return new JumpI(op, builder.var(value), builder.var(compare), destIndex); }
        }
    "#;

    let raw = scan_raw_statements("158.1", source).expect("scan succeeds");
    let manifest = derive_semantic_manifest(&raw);

    assert_eq!(manifest.version, "158.1");

    let set = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "set")
        .expect("set override exists");
    assert_eq!(set.emit, ["set", "$target", "$source"]);
    assert_eq!(set.receiver, "target");
    assert_eq!(set.inputs, ["source"]);
    assert!(set.outputs.is_empty());

    let op_add = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "op_add")
        .expect("op_add override exists");
    assert_eq!(op_add.emit, ["op", "add", "$out", "$lhs", "$rhs"]);
    assert_eq!(op_add.outputs, ["out"]);

    assert!(manifest
        .instructions
        .iter()
        .any(|instruction| instruction.rust_name == "op_not"));
    assert!(manifest
        .instructions
        .iter()
        .any(|instruction| instruction.rust_name == "jump_equal"));
    assert!(manifest
        .instructions
        .iter()
        .any(|instruction| instruction.rust_name == "jump_always"));

    let read = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "read")
        .expect("generic read exists");
    assert_eq!(read.emit, ["read", "$output", "$target", "$address"]);
    assert!(read.receiver.is_empty());
    assert_eq!(read.inputs, ["target", "address"]);
    assert_eq!(read.outputs, ["output"]);

    let print = manifest
        .instructions
        .iter()
        .find(|instruction| instruction.rust_name == "print")
        .expect("generic print exists");
    assert_eq!(print.emit, ["print", "$value"]);
    assert_eq!(print.inputs, ["value"]);
    assert!(print.outputs.is_empty());

    assert!(!manifest
        .instructions
        .iter()
        .any(|instruction| instruction.rust_name == "op"));
    assert!(!manifest
        .instructions
        .iter()
        .any(|instruction| instruction.rust_name == "jump"));
}
