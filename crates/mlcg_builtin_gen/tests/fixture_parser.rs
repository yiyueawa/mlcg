use mlcg_builtin_gen::fixture_parser::parse_fixture_manifest;

#[test]
fn fixture_parser_outputs_manifest_toml() {
    let input = include_str!("fixtures/v158_1_logic.txt");
    let toml_text = parse_fixture_manifest("158.1", input).expect("fixture parses");
    let value: toml::Value = toml::from_str(&toml_text).expect("generated TOML parses");
    let instructions = value["instructions"]
        .as_array()
        .expect("instructions are present");

    assert_eq!(value["version"].as_str(), Some("158.1"));
    assert!(toml_text.contains("rust_name = \"op_add\""));
    assert!(toml_text.contains("rust_name = \"op_not\""));
    assert!(toml_text.contains("rust_name = \"jump_always\""));
    assert!(toml_text.contains("rust_name = \"jump_equal\""));

    let op_not = instructions
        .iter()
        .find(|entry| entry["rust_name"].as_str() == Some("op_not"))
        .expect("op_not instruction exists");
    let emit: Vec<_> = op_not["emit"]
        .as_array()
        .expect("emit is an array")
        .iter()
        .map(|value| value.as_str().expect("emit token is string"))
        .collect();
    assert_eq!(emit, ["op", "not", "$out", "$input", "0"]);
}
