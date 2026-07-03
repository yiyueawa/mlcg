use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use snafu::ResultExt;

use crate::{
    error::{GenerateError, ReadSourceSnafu},
    manifest::Manifest,
    raw_statement::{scan_raw_enum_variants, scan_raw_statements, RawEnum, RawStatementManifest},
    semantic_manifest::derive_semantic_manifest,
};

pub fn parse_cached_mindustry(version: &str, cache_path: &Path) -> Result<Manifest, GenerateError> {
    let raw_manifest = scan_cached_mindustry_raw(version, cache_path)?;
    Ok(derive_semantic_manifest(&raw_manifest))
}

pub fn scan_cached_mindustry_raw(
    version: &str,
    cache_path: &Path,
) -> Result<RawStatementManifest, GenerateError> {
    let statements_path = cache_path.join("core/src/mindustry/logic/LStatements.java");
    let statements = std::fs::read_to_string(&statements_path).context(ReadSourceSnafu {
        path: statements_path,
    })?;

    let mut manifest = scan_raw_statements(version, &statements)?;
    manifest.enums = scan_manifest_enums(&manifest, cache_path)?;
    Ok(manifest)
}

fn scan_manifest_enums(
    manifest: &RawStatementManifest,
    cache_path: &Path,
) -> Result<Vec<RawEnum>, GenerateError> {
    let enum_names: BTreeSet<_> = manifest
        .statements
        .iter()
        .flat_map(|statement| statement.fields.iter())
        .map(|field| field.ty.as_str())
        .filter(|ty| !matches!(*ty, "String" | "int" | "boolean" | "float" | "double"))
        .collect();
    let source_root = cache_path.join("core/src");
    let java_files = collect_java_files(&source_root)?;
    let mut enums = Vec::new();

    for enum_name in enum_names {
        for java_file in &java_files {
            let source = std::fs::read_to_string(java_file).context(ReadSourceSnafu {
                path: java_file.clone(),
            })?;
            if source.contains(&format!("enum {enum_name}")) {
                enums.push(scan_raw_enum_variants(enum_name, &source)?);
                break;
            }
        }
    }

    Ok(enums)
}

fn collect_java_files(root: &Path) -> Result<Vec<PathBuf>, GenerateError> {
    let mut files = Vec::new();
    collect_java_files_into(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_java_files_into(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), GenerateError> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).context(ReadSourceSnafu {
            path: path.to_path_buf(),
        })? {
            let entry = entry.context(ReadSourceSnafu {
                path: path.to_path_buf(),
            })?;
            collect_java_files_into(&entry.path(), files)?;
        }
    } else if path
        .extension()
        .is_some_and(|extension| extension == "java")
    {
        files.push(path.to_path_buf());
    }
    Ok(())
}
