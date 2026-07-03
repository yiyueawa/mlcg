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

[[instructions]]
family = "multi"
variant = "multi"
rust_name = "multi"
emit = ["multi", "$outA", "$outB", "$input"]
receiver = ""
inputs = ["input"]
outputs = ["outA", "outB"]

[[instructions]]
family = "multi_recv"
variant = "multi_recv"
rust_name = "multi_recv"
emit = ["multi_recv", "$outA", "$outB", "$target", "$input"]
receiver = "target"
inputs = ["input"]
outputs = ["outA", "outB"]

[[instructions]]
family = "recv_out"
variant = "recv_out"
rust_name = "recv_out"
emit = ["recv_out", "$out", "$target", "$input"]
receiver = "target"
inputs = ["input"]
outputs = ["out"]

[[instructions]]
family = "keywords"
variant = "keywords"
rust_name = "keywords"
emit = ["keywords", "$loop", "$async", "$type", "$out"]
receiver = ""
inputs = ["loop", "async", "type"]
outputs = ["out"]
"#;
    fs::write(&manifest_path, manifest).expect("write manifest");
    let trybuild_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/tests/trybuild/mlcg_builtin_macros/tests/basic_manifest.toml");
    fs::create_dir_all(trybuild_manifest_path.parent().expect("manifest parent"))
        .expect("create trybuild manifest dir");
    fs::write(&trybuild_manifest_path, manifest).expect("write trybuild manifest");

    let collision_manifest = r#"
version = "fixture"

[[instructions]]
family = "fixture"
variant = "foo"
rust_name = "foo"
emit = ["foo", "$out"]
receiver = ""
inputs = []
outputs = ["out"]

[[instructions]]
family = "fixture"
variant = "foo_into"
rust_name = "foo_into"
emit = ["foo_into"]
receiver = ""
inputs = []
outputs = []
"#;
    let collision_manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/method_collision_manifest.toml");
    fs::write(&collision_manifest_path, collision_manifest).expect("write collision manifest");
    let trybuild_collision_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../target/tests/trybuild/mlcg_builtin_macros/tests/method_collision_manifest.toml",
    );
    fs::write(&trybuild_collision_manifest_path, collision_manifest)
        .expect("write trybuild collision manifest");

    let invalid_manifest = r#"
version = "fixture"

[[instructions]]
family = "broken"
"#;
    let invalid_manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/invalid_manifest.toml");
    fs::write(&invalid_manifest_path, invalid_manifest).expect("write invalid manifest");
    let trybuild_invalid_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/tests/trybuild/mlcg_builtin_macros/tests/invalid_manifest.toml");
    fs::write(&trybuild_invalid_manifest_path, invalid_manifest)
        .expect("write trybuild invalid manifest");

    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_macro_basic.rs");
    t.compile_fail("tests/ui/fail_method_collision.rs");
    t.compile_fail("tests/ui/fail_missing_manifest.rs");
    t.compile_fail("tests/ui/fail_invalid_manifest.rs");
}
