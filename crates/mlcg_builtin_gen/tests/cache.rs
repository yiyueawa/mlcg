use mlcg_builtin_gen::cache::{default_cache_path, validate_mindustry_cache};

#[test]
fn default_cache_path_uses_target_cache() {
    let path = default_cache_path("158.1");
    assert!(path.ends_with("target/mlcg-cache/mindustry/v158.1"));
}

#[test]
fn validation_reports_missing_expected_files() {
    let temp = tempfile::tempdir().expect("temp dir");
    let error = validate_mindustry_cache(temp.path()).expect_err("empty cache is invalid");
    assert!(error.to_string().contains("LStatements.java"));
}
