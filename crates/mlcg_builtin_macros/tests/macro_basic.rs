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

    let helper_collision_manifest = r#"
version = "fixture"

[[instructions]]
family = "fixture"
variant = "arg"
rust_name = "arg"
emit = ["arg"]
receiver = ""
inputs = []
outputs = []
"#;
    let helper_collision_manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/helper_collision_manifest.toml");
    fs::write(&helper_collision_manifest_path, helper_collision_manifest)
        .expect("write helper collision manifest");
    let trybuild_helper_collision_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../target/tests/trybuild/mlcg_builtin_macros/tests/helper_collision_manifest.toml",
    );
    fs::write(
        &trybuild_helper_collision_manifest_path,
        helper_collision_manifest,
    )
    .expect("write trybuild helper collision manifest");

    let output_field_collision_manifest = r#"
version = "fixture"

[[instructions]]
family = "fixture"
variant = "multi"
rust_name = "multi"
emit = ["multi"]
receiver = ""
inputs = []
outputs = ["type", "arg_type"]
"#;
    let output_field_collision_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/output_field_collision_manifest.toml");
    fs::write(
        &output_field_collision_manifest_path,
        output_field_collision_manifest,
    )
    .expect("write output field collision manifest");
    let trybuild_output_field_collision_manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../target/tests/trybuild/mlcg_builtin_macros/tests/output_field_collision_manifest.toml",
        );
    fs::write(
        &trybuild_output_field_collision_manifest_path,
        output_field_collision_manifest,
    )
    .expect("write trybuild output field collision manifest");

    let std_trait_name_manifest = r#"
version = "fixture"

[[instructions]]
family = "fixture"
variant = "clone"
rust_name = "clone"
emit = ["clone", "$outA", "$outB"]
receiver = ""
inputs = []
outputs = ["outA", "outB"]
"#;
    let std_trait_name_manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/std_trait_name_manifest.toml");
    fs::write(&std_trait_name_manifest_path, std_trait_name_manifest)
        .expect("write std trait name manifest");
    let trybuild_std_trait_name_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/tests/trybuild/mlcg_builtin_macros/tests/std_trait_name_manifest.toml");
    fs::write(
        &trybuild_std_trait_name_manifest_path,
        std_trait_name_manifest,
    )
    .expect("write trybuild std trait name manifest");

    let std_convert_trait_name_manifest = r#"
version = "fixture"

[[instructions]]
family = "fixture"
variant = "from"
rust_name = "from"
emit = ["from", "$out", "$input"]
receiver = ""
inputs = ["input"]
outputs = ["out"]

[[instructions]]
family = "fixture"
variant = "into"
rust_name = "into"
emit = ["into", "$out", "$input"]
receiver = ""
inputs = ["input"]
outputs = ["out"]
"#;
    let std_convert_trait_name_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/std_convert_trait_name_manifest.toml");
    fs::write(
        &std_convert_trait_name_manifest_path,
        std_convert_trait_name_manifest,
    )
    .expect("write std convert trait name manifest");
    let trybuild_std_convert_trait_name_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(
            "../../target/tests/trybuild/mlcg_builtin_macros/tests/std_convert_trait_name_manifest.toml",
        );
    fs::write(
        &trybuild_std_convert_trait_name_manifest_path,
        std_convert_trait_name_manifest,
    )
    .expect("write trybuild std convert trait name manifest");

    let std_collection_type_name_manifest = r#"
version = "fixture"

[[instructions]]
family = "fixture"
variant = "string"
rust_name = "string"
emit = ["string", "$input"]
receiver = ""
inputs = ["input"]
outputs = []

[[instructions]]
family = "fixture"
variant = "vec"
rust_name = "vec"
emit = ["vec", "$input"]
receiver = ""
inputs = ["input"]
outputs = []
"#;
    let std_collection_type_name_manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/std_collection_type_name_manifest.toml");
    fs::write(
        &std_collection_type_name_manifest_path,
        std_collection_type_name_manifest,
    )
    .expect("write std collection type name manifest");
    let trybuild_std_collection_type_name_manifest_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../target/tests/trybuild/mlcg_builtin_macros/tests/std_collection_type_name_manifest.toml",
        );
    fs::write(
        &trybuild_std_collection_type_name_manifest_path,
        std_collection_type_name_manifest,
    )
    .expect("write trybuild std collection type name manifest");

    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_macro_basic.rs");
    t.pass("tests/ui/pass_std_trait_name.rs");
    t.pass("tests/ui/pass_std_convert_trait_names.rs");
    t.pass("tests/ui/pass_std_collection_type_names.rs");
    t.compile_fail("tests/ui/fail_method_collision.rs");
    t.compile_fail("tests/ui/fail_missing_manifest.rs");
    t.compile_fail("tests/ui/fail_invalid_manifest.rs");
    t.compile_fail("tests/ui/fail_raw_output_argument.rs");
    t.compile_fail("tests/ui/fail_helper_collision.rs");
    t.compile_fail("tests/ui/fail_output_field_collision.rs");
}
