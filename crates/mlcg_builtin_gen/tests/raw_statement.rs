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

#[test]
fn scans_enum_variants_from_source() {
    let source = r#"
        public enum LogicOp{
            add("+", true),
            sub("-", true),
            not("!", false);
        }
    "#;

    let variants = mlcg_builtin_gen::raw_statement::scan_raw_enum_variants("LogicOp", source)
        .expect("scan succeeds");

    assert_eq!(variants.name, "LogicOp");
    assert_eq!(variants.variants, ["add", "sub", "not"]);
}

#[test]
fn scans_direct_superclass_fields_for_registered_statements() {
    let source = r#"
        @RegisterStatement("radar")
        public static class RadarStatement extends LStatement{
            public RadarTarget target1 = RadarTarget.enemy, target2 = RadarTarget.any;
            public String radar = "turret1", output = "result";
            public static String ignoredStatic = "x";
            public transient String ignoredTransient = "y";
            @Override public LInstruction build(LAssembler builder){
                return new RadarI(target1, target2, builder.var(radar), builder.var(output));
            }
        }

        @RegisterStatement("uradar")
        public static class UnitRadarStatement extends RadarStatement{
            public UnitRadarStatement(){
                radar = "0";
            }
            @Override public LInstruction build(LAssembler builder){
                return new RadarI(target1, target2, builder.var("@unit"), builder.var(output));
            }
        }
    "#;

    let manifest = scan_raw_statements("158.1", source).expect("scan succeeds");
    let uradar = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "uradar")
        .expect("uradar exists");
    let fields: Vec<_> = uradar
        .fields
        .iter()
        .map(|field| field.name.as_str())
        .collect();

    assert_eq!(fields, ["target1", "target2", "radar", "output"]);
    let radar = uradar
        .fields
        .iter()
        .find(|field| field.name == "radar")
        .expect("radar field exists");
    assert_eq!(radar.default.as_deref(), Some("0"));
    assert_eq!(uradar.ignored_fields, ["radar"]);
}

#[test]
fn scans_field_declarations_after_methods_or_constructors() {
    let source = r#"
        @RegisterStatement("late")
        public static class LateFieldStatement extends LStatement{
            public LateFieldStatement(){
                first = "changed";
            }

            @Override public LInstruction build(LAssembler builder){
                return new LateI(builder.var(first), builder.var(second));
            }

            public String first = "a", second = "b";
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let late = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "late")
        .expect("late exists");
    let fields: Vec<_> = late
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field.default.as_deref()))
        .collect();

    assert_eq!(fields, [("first", Some("changed")), ("second", Some("b"))]);
}

#[test]
fn scans_instruction_type_from_linstruction_build_method_only() {
    let source = r#"
        @RegisterStatement("real")
        public static class RealStatement extends LStatement{
            public String value = "x";

            public LInstruction helper(){
                return new WrongI();
            }

            @Override public LInstruction build(LAssembler builder){
                return new RealI(builder.var(value));
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let real = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "real")
        .expect("real exists");

    assert_eq!(real.instruction.as_deref(), Some("RealI"));
}
