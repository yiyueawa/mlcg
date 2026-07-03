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
fn infers_constant_true_enum_lambda_as_zero_arity() {
    let source = r#"
        public enum ConditionOp{
            equal("==", (a, b) -> Math.abs(a - b) < 0.000001),
            always("always", (a, b) -> true);
        }
    "#;

    let variants = mlcg_builtin_gen::raw_statement::scan_raw_enum_variants("ConditionOp", source)
        .expect("scan succeeds");

    assert_eq!(variants.arities.get("equal"), Some(&2));
    assert_eq!(variants.arities.get("always"), Some(&0));
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
fn scans_transitive_superclass_fields_for_registered_statements() {
    let source = r#"
        public static class BaseRadarStatement extends LStatement{
            public RadarTarget target = RadarTarget.enemy;
            public String radar = "turret1";
        }

        public static class MidRadarStatement extends BaseRadarStatement{
            public String sort = "distance";
        }

        @RegisterStatement("deep_radar")
        public static class DeepRadarStatement extends MidRadarStatement{
            public String output = "result";
            public DeepRadarStatement(){
                radar = "0";
            }
            @Override public LInstruction build(LAssembler builder){
                return new RadarI(target, builder.var(radar), builder.var(sort), builder.var(output));
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let deep_radar = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "deep_radar")
        .expect("deep_radar exists");
    let fields: Vec<_> = deep_radar
        .fields
        .iter()
        .map(|field| {
            (
                field.ty.as_str(),
                field.name.as_str(),
                field.default.as_deref(),
            )
        })
        .collect();

    assert_eq!(
        fields,
        [
            ("String", "output", Some("result")),
            ("String", "sort", Some("distance")),
            ("RadarTarget", "target", Some("RadarTarget.enemy")),
            ("String", "radar", Some("0")),
        ]
    );
    assert!(deep_radar.ignored_fields.is_empty());
}

#[test]
fn applies_transitive_superclass_constructor_defaults() {
    let source = r#"
        public static class BaseRadarStatement extends LStatement{
            public RadarTarget target = RadarTarget.enemy;
            public String radar = "turret1";
            public BaseRadarStatement(){
                radar = "base";
            }
        }

        public static class MidRadarStatement extends BaseRadarStatement{
            public String sort = "distance";
            public MidRadarStatement(){
                radar = "mid";
                sort = "health";
            }
        }

        @RegisterStatement("deep_radar")
        public static class DeepRadarStatement extends MidRadarStatement{
            public String output = "result";
            @Override public LInstruction build(LAssembler builder){
                return new RadarI(target, builder.var(radar), builder.var(sort), builder.var(output));
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let deep_radar = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "deep_radar")
        .expect("deep_radar exists");
    let fields: Vec<_> = deep_radar
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field.default.as_deref()))
        .collect();

    assert_eq!(
        fields,
        [
            ("output", Some("result")),
            ("sort", Some("health")),
            ("target", Some("RadarTarget.enemy")),
            ("radar", Some("mid")),
        ]
    );
}

#[test]
fn subclass_field_hides_superclass_field_with_same_name() {
    let source = r#"
        public static class BaseStatement extends LStatement{
            public String value = "base";
        }

        @RegisterStatement("hidden")
        public static class HiddenStatement extends BaseStatement{
            public String value = "child", output = "result";
            @Override public LInstruction build(LAssembler builder){
                return new HiddenI(builder.var(value), builder.var(output));
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let hidden = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "hidden")
        .expect("hidden exists");
    let fields: Vec<_> = hidden
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field.default.as_deref()))
        .collect();

    assert_eq!(
        fields,
        [("value", Some("child")), ("output", Some("result"))]
    );
}

#[test]
fn ignores_fields_that_only_appear_inside_build_string_literals() {
    let source = r#"
        @RegisterStatement("literal")
        public static class LiteralStatement extends LStatement{
            public String target = "block1", output = "result";

            @Override public LInstruction build(LAssembler builder){
                return new LiteralI(builder.var("@target"), builder.var(output));
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let literal = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "literal")
        .expect("literal exists");

    assert_eq!(literal.ignored_fields, ["target"]);
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
fn scans_comma_separated_fields_with_commas_inside_string_defaults() {
    let source = r#"
        @RegisterStatement("comma")
        public static class CommaStatement extends LStatement{
            public String message = "hello, world", target = "cell1";

            @Override public LInstruction build(LAssembler builder){
                return new CommaI(builder.var(message), builder.var(target));
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let comma = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "comma")
        .expect("comma exists");
    let fields: Vec<_> = comma
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field.default.as_deref()))
        .collect();

    assert_eq!(
        fields,
        [("message", Some("hello, world")), ("target", Some("cell1"))]
    );
}

#[test]
fn scans_constructor_assignment_defaults_with_semicolons_inside_strings() {
    let source = r#"
        @RegisterStatement("message")
        public static class MessageStatement extends LStatement{
            public String message = "default", output = "result";

            public MessageStatement(){
                message = "hello; world";
            }

            @Override public LInstruction build(LAssembler builder){
                return new MessageI(builder.var(message), builder.var(output));
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let message = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "message")
        .expect("message exists");
    let fields: Vec<_> = message
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field.default.as_deref()))
        .collect();

    assert_eq!(
        fields,
        [
            ("message", Some("hello; world")),
            ("output", Some("result"))
        ]
    );
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

#[test]
fn does_not_confuse_build_with_build_prefixed_helper_methods() {
    let source = r#"
        @RegisterStatement("real")
        public static class RealStatement extends LStatement{
            public String value = "x";

            public LInstruction buildHelper(){
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

#[test]
fn scans_category_from_lcategory_category_method_only() {
    let source = r#"
        @RegisterStatement("cat")
        public static class CategoryStatement extends LStatement{
            public LCategory helper(){
                return LCategory.wrong;
            }

            @Override public LInstruction build(LAssembler builder){
                return new CatI();
            }

            @Override public LCategory category(){
                return LCategory.control;
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let cat = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "cat")
        .expect("cat exists");

    assert_eq!(cat.category.as_deref(), Some("control"));
}

#[test]
fn does_not_confuse_category_with_category_prefixed_helper_methods() {
    let source = r#"
        @RegisterStatement("cat")
        public static class CategoryStatement extends LStatement{
            public LCategory categoryHelper(){
                return LCategory.wrong;
            }

            @Override public LInstruction build(LAssembler builder){
                return new CatI();
            }

            @Override public LCategory category(){
                return LCategory.control;
            }
        }
    "#;

    let manifest = scan_raw_statements("fixture", source).expect("scan succeeds");
    let cat = manifest
        .statements
        .iter()
        .find(|statement| statement.name == "cat")
        .expect("cat exists");

    assert_eq!(cat.category.as_deref(), Some("control"));
}
