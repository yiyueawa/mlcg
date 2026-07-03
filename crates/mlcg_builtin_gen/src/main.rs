use std::{env, fs, path::PathBuf};

use mlcg_builtin_gen::{
    cache::ensure_mindustry_cache, fixture_parser::parse_fixture_manifest,
    source_parser::parse_cached_mindustry,
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
            fs::write(output, manifest.to_toml()?)?;
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
            return Err("usage: mlcg_builtin_gen <fetch|fixture> ...".into());
        }
    }
    Ok(())
}
