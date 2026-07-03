use std::{env, fs, path::PathBuf};

use mlcg_builtin_gen::fixture_parser::parse_fixture_manifest;

fn main() {
    let mut args = env::args().skip(1);
    let version = args.next().unwrap_or_else(|| "158.1".to_string());
    let input = args
        .next()
        .map(PathBuf::from)
        .expect("usage: mlcg_builtin_gen <version> <fixture-input> <output-toml>");
    let output = args
        .next()
        .map(PathBuf::from)
        .expect("usage: mlcg_builtin_gen <version> <fixture-input> <output-toml>");

    let source = fs::read_to_string(&input).expect("read input fixture");
    let manifest = parse_fixture_manifest(&version, &source).expect("parse fixture");
    fs::write(&output, manifest).expect("write manifest");
}
