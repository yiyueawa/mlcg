use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use snafu::{ensure, ResultExt};

use crate::{
    error::{GenerateError, ReadSourceSnafu, RequiredItemMissingSnafu},
    manifest::{Instruction, Manifest},
    raw_statement::{scan_raw_enum_variants, scan_raw_statements, RawEnum, RawStatementManifest},
    semantic_manifest::derive_semantic_manifest,
};

pub fn parse_representative_manifest(
    version: &str,
    l_statements: &str,
    logic_op: &str,
    condition_op: &str,
) -> Result<Manifest, GenerateError> {
    ensure!(
        l_statements.contains("@RegisterStatement(\"set\")"),
        RequiredItemMissingSnafu {
            item: "set statement registration"
        }
    );
    ensure!(
        contains_enum_variant(logic_op, "add"),
        RequiredItemMissingSnafu {
            item: "LogicOp.add"
        }
    );
    ensure!(
        contains_enum_variant(logic_op, "not"),
        RequiredItemMissingSnafu {
            item: "LogicOp.not"
        }
    );
    ensure!(
        contains_enum_variant(condition_op, "equal"),
        RequiredItemMissingSnafu {
            item: "ConditionOp.equal"
        }
    );
    ensure!(
        contains_enum_variant(condition_op, "always"),
        RequiredItemMissingSnafu {
            item: "ConditionOp.always"
        }
    );

    Ok(Manifest {
        version: version.to_string(),
        instructions: vec![
            Instruction {
                family: "set".to_string(),
                variant: "set".to_string(),
                rust_name: "set".to_string(),
                emit: strings(["set", "$target", "$source"]),
                receiver: "target".to_string(),
                inputs: strings(["source"]),
                outputs: Vec::new(),
                labels: Vec::new(),
            },
            Instruction {
                family: "op".to_string(),
                variant: "add".to_string(),
                rust_name: "op_add".to_string(),
                emit: strings(["op", "add", "$out", "$lhs", "$rhs"]),
                receiver: String::new(),
                inputs: strings(["lhs", "rhs"]),
                outputs: strings(["out"]),
                labels: Vec::new(),
            },
            Instruction {
                family: "op".to_string(),
                variant: "not".to_string(),
                rust_name: "op_not".to_string(),
                emit: strings(["op", "not", "$out", "$input", "0"]),
                receiver: String::new(),
                inputs: strings(["input"]),
                outputs: strings(["out"]),
                labels: Vec::new(),
            },
            Instruction {
                family: "jump".to_string(),
                variant: "equal".to_string(),
                rust_name: "jump_equal".to_string(),
                emit: strings(["jump", "$label", "equal", "$lhs", "$rhs"]),
                receiver: String::new(),
                inputs: strings(["label", "lhs", "rhs"]),
                outputs: Vec::new(),
                labels: strings(["label"]),
            },
            Instruction {
                family: "jump".to_string(),
                variant: "always".to_string(),
                rust_name: "jump_always".to_string(),
                emit: strings(["jump", "$label", "always", "0", "0"]),
                receiver: String::new(),
                inputs: strings(["label"]),
                outputs: Vec::new(),
                labels: strings(["label"]),
            },
        ],
    })
}

fn contains_enum_variant(source: &str, variant: &str) -> bool {
    let pattern = format!("{variant}(");
    source.contains(&pattern)
}

fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.into_iter().map(ToString::to_string).collect()
}

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
