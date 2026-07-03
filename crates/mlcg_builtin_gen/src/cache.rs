use std::{
    path::{Path, PathBuf},
    process::Command,
};

use snafu::ensure;

use crate::error::{GenerateError, GitCommandSnafu, MissingCacheFileSnafu};

const MINDUSTRY_REMOTE: &str = "https://github.com/Anuken/Mindustry.git";
const EXPECTED_FILES: &[&str] = &[
    "core/src/mindustry/logic/LStatements.java",
    "core/src/mindustry/logic/LogicOp.java",
    "core/src/mindustry/logic/ConditionOp.java",
];

pub fn default_cache_path(version: &str) -> PathBuf {
    PathBuf::from("target")
        .join("mlcg-cache")
        .join("mindustry")
        .join(format!("v{version}"))
}

pub fn validate_mindustry_cache(path: &Path) -> Result<(), GenerateError> {
    for expected in EXPECTED_FILES {
        let source_path = path.join(expected);
        ensure!(
            source_path.is_file(),
            MissingCacheFileSnafu { path: source_path }
        );
    }
    Ok(())
}

pub fn ensure_mindustry_cache(version: &str) -> Result<PathBuf, GenerateError> {
    let path = default_cache_path(version);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| GenerateError::ReadSource {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let tag = format!("v{version}");
        let status = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("--branch")
            .arg(&tag)
            .arg(MINDUSTRY_REMOTE)
            .arg(&path)
            .status()
            .map_err(|source| GenerateError::ReadSource {
                path: PathBuf::from("git"),
                source,
            })?;
        ensure!(
            status.success(),
            GitCommandSnafu {
                command: format!(
                    "git clone --depth 1 --branch {tag} {MINDUSTRY_REMOTE} {}",
                    path.display()
                ),
                status: status.to_string(),
            }
        );
    }
    validate_mindustry_cache(&path)?;
    Ok(path)
}
