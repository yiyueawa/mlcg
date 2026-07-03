use mlcg_builtin_gen::raw_statement::scan_raw_statements;

#[test]
fn scans_registered_statement_classes() {
    let source = r#"
        @RegisterStatement("read")
        public static class ReadStatement extends LStatement{
            public String output = "result", target = "cell1", address = "0";
            public static String ignoredStatic = "x";
            public transient String ignoredTransient = "y";
            @Override public LInstruction build(LAssembler builder){
                return new ReadI(builder.var(target), builder.var(address), builder.var(output));
            }
            @Override public LCategory category(){ return LCategory.io; }
        }

        @RegisterStatement("set")
        public static class SetStatement extends LStatement{
            public String to = "result";
            public String from = "0";
            @Override public LInstruction build(LAssembler builder){ return new SetI(builder.var(from), builder.var(to)); }
            @Override public LCategory category(){ return LCategory.operation; }
        }

        @RegisterStatement("op")
        public static class OperationStatement extends LStatement{
            public LogicOp op = LogicOp.add;
            public String dest = "result", a = "a", b = "b";
            @Override public LInstruction build(LAssembler builder){ return new OpI(op,builder.var(a), builder.var(b), builder.var(dest)); }
            @Override public LCategory category(){ return LCategory.operation; }
        }

        @RegisterStatement("jump")
        public static class JumpStatement extends LStatement{
            public transient Object dest;
            public int destIndex;
            public ConditionOp op = ConditionOp.notEqual;
            public String value = "x", compare = "false";
            @Override public LInstruction build(LAssembler builder){ return new JumpI(op, builder.var(value), builder.var(compare), destIndex); }
            @Override public LCategory category(){ return LCategory.control; }
        }
    "#;

    let manifest = scan_raw_statements("158.1", source).expect("scan succeeds");

    assert_eq!(manifest.version, "158.1");
    assert_eq!(manifest.statements.len(), 4);

    let read = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "read")
        .expect("read exists");
    assert_eq!(read.class, "ReadStatement");
    assert_eq!(read.instruction.as_deref(), Some("ReadI"));
    assert_eq!(read.category.as_deref(), Some("io"));
    assert_eq!(read.fields.len(), 3);
    assert!(read
        .fields
        .iter()
        .any(|field| field.name == "output" && field.default.as_deref() == Some("result")));
    assert!(!read
        .fields
        .iter()
        .any(|field| field.name == "ignoredStatic"));
    assert!(!read
        .fields
        .iter()
        .any(|field| field.name == "ignoredTransient"));

    let jump = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "jump")
        .expect("jump exists");
    assert_eq!(jump.instruction.as_deref(), Some("JumpI"));
    assert_eq!(jump.category.as_deref(), Some("control"));
    assert!(jump
        .fields
        .iter()
        .any(|field| field.ty == "int" && field.name == "destIndex"));
    assert!(jump
        .fields
        .iter()
        .any(|field| field.ty == "ConditionOp" && field.name == "op"));
}

#[test]
fn ignores_commented_register_statement_annotations() {
    let source = r##"
        //@RegisterStatement("#")
        public static class CommentStatement extends LStatement{
            public String comment = "";
        }

        @RegisterStatement("noop")
        public static class InvalidStatement extends LStatement{
            @Override public LInstruction build(LAssembler builder){ return new NoopI(); }
        }
    "##;

    let manifest = scan_raw_statements("158.1", source).expect("scan succeeds");

    assert_eq!(manifest.statements.len(), 1);
    assert_eq!(manifest.statements[0].name, "noop");
}
