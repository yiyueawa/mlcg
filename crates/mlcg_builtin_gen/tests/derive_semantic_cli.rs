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
    assert!(semantic.contains("\"$target\","));
    assert!(semantic.contains("\"$source\","));
    assert!(semantic.contains("rust_name = \"print\""));
    assert!(semantic.contains("\"print\","));
    assert!(semantic.contains("\"$value\","));
}
