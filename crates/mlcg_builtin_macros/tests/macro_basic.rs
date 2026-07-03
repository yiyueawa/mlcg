use std::{fs, path::PathBuf};

#[test]
fn macro_generates_set_and_op_add() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/basic_manifest.toml");
    let manifest = r#"
version = "fixture"

[[instructions]]
family = "set"
variant = "set"
rust_name = "set"
emit = ["set", "$target", "$source"]
receiver = "target"
inputs = ["source"]
outputs = []

[[instructions]]
family = "op"
variant = "add"
rust_name = "op_add"
emit = ["op", "add", "$out", "$lhs", "$rhs"]
receiver = ""
inputs = ["lhs", "rhs"]
outputs = ["out"]
"#;
    fs::write(&manifest_path, manifest).expect("write manifest");
    let trybuild_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/tests/trybuild/mlcg_builtin_macros/tests/basic_manifest.toml");
    fs::create_dir_all(trybuild_manifest_path.parent().expect("manifest parent"))
        .expect("create trybuild manifest dir");
    fs::write(&trybuild_manifest_path, manifest).expect("write trybuild manifest");

    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_macro_basic.rs");
}
