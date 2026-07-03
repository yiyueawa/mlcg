use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use mlcg_builtin_gen::{
    cache::ensure_mindustry_cache,
    fixture_parser::parse_fixture_manifest,
    raw_statement::{scan_raw_enum_variants, scan_raw_statements, RawEnum, RawStatementManifest},
    semantic_manifest::derive_semantic_manifest,
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
        Some("scan-statements") => {
            let version = args
                .next()
                .ok_or("usage: mlcg_builtin_gen scan-statements <version> <output-toml>")?;
            let output = args
                .next()
                .map(PathBuf::from)
                .ok_or("usage: mlcg_builtin_gen scan-statements <version> <output-toml>")?;
            let cache = ensure_mindustry_cache(&version)?;
            let statements_path = cache.join("core/src/mindustry/logic/LStatements.java");
            let statements = fs::read_to_string(&statements_path)?;
            let mut manifest = scan_raw_statements(&version, &statements)?;
            manifest.enums = scan_manifest_enums(&manifest, &cache)?;
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
            return Err(
                "usage: mlcg_builtin_gen <fetch|fixture|scan-statements|derive-semantic> ..."
                    .into(),
            );
        }
    }
    Ok(())
}

fn scan_manifest_enums(
    manifest: &RawStatementManifest,
    cache: &Path,
) -> Result<Vec<RawEnum>, Box<dyn std::error::Error>> {
    let enum_names: BTreeSet<_> = manifest
        .statements
        .iter()
        .flat_map(|statement| statement.fields.iter())
        .map(|field| field.ty.as_str())
        .filter(|ty| !matches!(*ty, "String" | "int" | "boolean" | "float" | "double"))
        .collect();
    let source_root = cache.join("core/src");
    let java_files = collect_java_files(&source_root)?;
    let mut enums = Vec::new();

    for enum_name in enum_names {
        for java_file in &java_files {
            let source = fs::read_to_string(java_file)?;
            if source.contains(&format!("enum {enum_name}")) {
                enums.push(scan_raw_enum_variants(enum_name, &source)?);
                break;
            }
        }
    }

    Ok(enums)
}

fn collect_java_files(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    collect_java_files_into(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_java_files_into(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            collect_java_files_into(&entry?.path(), files)?;
        }
    } else if path
        .extension()
        .is_some_and(|extension| extension == "java")
    {
        files.push(path.to_path_buf());
    }
    Ok(())
}
