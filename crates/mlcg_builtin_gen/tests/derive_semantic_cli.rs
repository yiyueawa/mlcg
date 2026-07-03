use std::{fs, process::Command};

#[test]
fn derive_semantic_cli_writes_semantic_manifest_from_raw_toml() {
    let temp = tempfile::tempdir().expect("temp dir");
    let raw_path = temp.path().join("raw.toml");
    let semantic_path = temp.path().join("semantic.toml");
    fs::write(
        &raw_path,
        r#"
version = "fixture"

[[statements]]
name = "set"
class = "SetStatement"
instruction = "SetI"

[[statements.fields]]
ty = "String"
name = "to"
default = "result"

[[statements.fields]]
ty = "String"
name = "from"
default = "0"

[[statements]]
name = "print"
class = "PrintStatement"
instruction = "PrintI"

[[statements.fields]]
ty = "String"
name = "value"
default = "message"
"#,
    )
    .expect("write raw manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_mlcg_builtin_gen"))
        .arg("derive-semantic")
        .arg(&raw_path)
        .arg(&semantic_path)
        .output()
        .expect("run derive-semantic");

    assert!(
        output.status.success(),
        "derive-semantic failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let semantic = fs::read_to_string(&semantic_path).expect("read semantic manifest");
    assert!(semantic.contains("rust_name = \"set\""));
    assert!(semantic.contains("\"set\","));
    assert!(semantic.contains("\"$to\","));
    assert!(semantic.contains("\"$from\","));
    assert!(semantic.contains("rust_name = \"print\""));
    assert!(semantic.contains("\"print\","));
    assert!(semantic.contains("\"$value\","));
}

#[test]
fn derive_semantic_cli_rejects_manifest_that_would_generate_method_collision() {
    let temp = tempfile::tempdir().expect("temp dir");
    let raw_path = temp.path().join("raw.toml");
    let semantic_path = temp.path().join("semantic.toml");
    fs::write(
        &raw_path,
        r#"
version = "fixture"

[[statements]]
name = "foo"
class = "FooStatement"
instruction = "FooI"

[[statements.fields]]
ty = "String"
name = "output"
default = "result"

[[statements]]
name = "foo_into"
class = "FooIntoStatement"
instruction = "FooIntoI"
fields = []
"#,
    )
    .expect("write raw manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_mlcg_builtin_gen"))
        .arg("derive-semantic")
        .arg(&raw_path)
        .arg(&semantic_path)
        .output()
        .expect("run derive-semantic");

    assert!(
        !output.status.success(),
        "derive-semantic unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "generated processor method `foo_into` for instruction `foo_into` collides with instruction `foo`"
        ),
        "stderr did not explain generated API collision:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !semantic_path.exists(),
        "invalid semantic manifest should not be written"
    );
}

#[test]
fn derive_semantic_cli_rejects_selector_field_without_enum_definition() {
    let temp = tempfile::tempdir().expect("temp dir");
    let raw_path = temp.path().join("raw.toml");
    let semantic_path = temp.path().join("semantic.toml");
    fs::write(
        &raw_path,
        r#"
version = "fixture"

[[statements]]
name = "calc"
class = "CalcStatement"
instruction = "CalcI"

[[statements.fields]]
ty = "MissingOp"
name = "op"
default = "MissingOp.add"

[[statements.fields]]
ty = "String"
name = "dest"
default = "result"

[[statements.fields]]
ty = "String"
name = "a"
default = "a"

[[statements.fields]]
ty = "String"
name = "b"
default = "b"
"#,
    )
    .expect("write raw manifest");

    let output = Command::new(env!("CARGO_BIN_EXE_mlcg_builtin_gen"))
        .arg("derive-semantic")
        .arg(&raw_path)
        .arg(&semantic_path)
        .output()
        .expect("run derive-semantic");

    assert!(
        !output.status.success(),
        "derive-semantic unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("required source item not found: enum MissingOp"),
        "stderr did not explain missing selector enum:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !semantic_path.exists(),
        "invalid semantic manifest should not be written"
    );
}
