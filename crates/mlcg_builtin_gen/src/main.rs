use std::{env, fs, path::PathBuf};

use mlcg_builtin_gen::{
    cache::ensure_mindustry_cache,
    fixture_parser::parse_fixture_manifest,
    generated_api::validate_generated_rust_api_symbols,
    manifest::Manifest,
    raw_statement::RawStatementManifest,
    semantic_manifest::derive_semantic_manifest,
    source_parser::{parse_cached_mindustry, scan_cached_mindustry_raw},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("fetch") => {
            let version = args
                .next()
                .ok_or("usage: mlcg_builtin_gen fetch <version> <output-toml>")?;
            let output = args
                .next()
                .map(PathBuf::from)
                .ok_or("usage: mlcg_builtin_gen fetch <version> <output-toml>")?;
            let cache = ensure_mindustry_cache(&version)?;
            let manifest = parse_cached_mindustry(&version, &cache)?;
            write_semantic_manifest(output, &manifest)?;
        }
        Some("scan-statements") => {
            let version = args
                .next()
                .ok_or("usage: mlcg_builtin_gen scan-statements <version> <output-toml>")?;
            let output = args
                .next()
                .map(PathBuf::from)
                .ok_or("usage: mlcg_builtin_gen scan-statements <version> <output-toml>")?;
            let cache = ensure_mindustry_cache(&version)?;
            let manifest = scan_cached_mindustry_raw(&version, &cache)?;
            fs::write(output, manifest.to_toml()?)?;
        }

        Some("derive-semantic") => {
            let input = args.next().map(PathBuf::from).ok_or(
                "usage: mlcg_builtin_gen derive-semantic <raw-input-toml> <semantic-output-toml>",
            )?;
            let output = args.next().map(PathBuf::from).ok_or(
                "usage: mlcg_builtin_gen derive-semantic <raw-input-toml> <semantic-output-toml>",
            )?;
            let raw_toml = fs::read_to_string(&input)?;
            let raw_manifest: RawStatementManifest = toml::from_str(&raw_toml)?;
            let manifest = derive_semantic_manifest(&raw_manifest);
            write_semantic_manifest(output, &manifest)?;
        }
        Some("fixture") => {
            let version = args
                .next()
                .ok_or("usage: mlcg_builtin_gen fixture <version> <fixture-input> <output-toml>")?;
            let input = args
                .next()
                .map(PathBuf::from)
                .ok_or("usage: mlcg_builtin_gen fixture <version> <fixture-input> <output-toml>")?;
            let output = args
                .next()
                .map(PathBuf::from)
                .ok_or("usage: mlcg_builtin_gen fixture <version> <fixture-input> <output-toml>")?;
            let source = fs::read_to_string(&input)?;
            let manifest = parse_fixture_manifest(&version, &source)?;
            fs::write(&output, manifest)?;
        }
        _ => {
            return Err(
                "usage: mlcg_builtin_gen <fetch|fixture|scan-statements|derive-semantic> ..."
                    .into(),
            );
        }
    }
    Ok(())
}

fn write_semantic_manifest(
    output: PathBuf,
    manifest: &Manifest,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_generated_rust_api_symbols(manifest)?;
    fs::write(output, manifest.to_toml()?)?;
    Ok(())
}
