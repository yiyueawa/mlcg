# Mindustry Fetch Parse Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend `mlcg_builtin_gen` so it can fetch Mindustry v158.1 into a local git cache and generate the representative builtin manifest from real source files.

**Architecture:** Keep normal tests offline by testing parser functions against source snippets. Add a small cache module that shells out to `git` for the real `fetch` command. Reuse the existing manifest schema and writer so generated output remains compatible with `mlcg_builtin_macros`.

**Tech Stack:** Rust, Cargo, SNAFU, serde/toml, `std::process::Command` for git.

---

## File structure

Create or modify:

```text
crates/mlcg_builtin_gen/src/error.rs          # generator semantic errors
crates/mlcg_builtin_gen/src/manifest.rs       # shared manifest structs and TOML writer
crates/mlcg_builtin_gen/src/fixture_parser.rs # adapt to shared manifest writer
crates/mlcg_builtin_gen/src/source_parser.rs  # real-source representative parser
crates/mlcg_builtin_gen/src/cache.rs          # git cache management
crates/mlcg_builtin_gen/src/lib.rs            # exports
crates/mlcg_builtin_gen/src/main.rs           # CLI fetch/fixture modes
crates/mlcg_builtin_gen/tests/source_parser.rs# offline parser tests
crates/mlcg_builtin_gen/tests/cache.rs        # cache validation tests without network
```

## Task 1: Introduce shared manifest model and generator errors

**Files:**
- Create: `crates/mlcg_builtin_gen/src/error.rs`
- Create: `crates/mlcg_builtin_gen/src/manifest.rs`
- Modify: `crates/mlcg_builtin_gen/src/lib.rs`
- Modify: `crates/mlcg_builtin_gen/src/fixture_parser.rs`
- Test: `crates/mlcg_builtin_gen/tests/fixture_parser.rs`

- [ ] **Step 1: Run existing fixture test as baseline**

Run:

```bash
cargo test -p mlcg_builtin_gen --test fixture_parser
```

Expected: PASS before refactor.

- [ ] **Step 2: Add shared error and manifest modules**

Create `src/error.rs` with a `GenerateError` enum using SNAFU variants for missing token, invalid key-value, TOML serialization, git command failure, missing cache file, source read failure, and required item missing.

Create `src/manifest.rs` with:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct Manifest {
    pub version: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Instruction {
    pub family: String,
    pub variant: String,
    pub rust_name: String,
    pub emit: Vec<String>,
    pub receiver: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

impl Manifest {
    pub fn to_toml(&self) -> Result<String, crate::error::GenerateError> { ... }
}
```

- [ ] **Step 3: Refactor fixture parser to use shared model**

Change `parse_fixture_manifest` to return `Result<String, GenerateError>` and build `manifest::Manifest` / `manifest::Instruction`.

- [ ] **Step 4: Export modules**

Update `src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod cache;
pub mod error;
pub mod fixture_parser;
pub mod manifest;
pub mod source_parser;
```

If `cache` and `source_parser` do not exist yet, create empty modules with `#![allow(dead_code)]` removed once implemented.

- [ ] **Step 5: Verify and commit**

```bash
cargo fmt --all
cargo test -p mlcg_builtin_gen --test fixture_parser
git add crates/mlcg_builtin_gen
git commit -m "refactor(gen): share manifest model"
```

## Task 2: Add offline real-source parser tests and implementation

**Files:**
- Create: `crates/mlcg_builtin_gen/src/source_parser.rs`
- Create: `crates/mlcg_builtin_gen/tests/source_parser.rs`

- [ ] **Step 1: Write failing parser tests**

Create tests that pass small Java snippets:

```rust
use mlcg_builtin_gen::source_parser::parse_representative_manifest;

#[test]
fn parses_required_representative_entries() {
    let statements = r#"@RegisterStatement("set") public static class SetStatement {}"#;
    let logic_op = r#"
        public enum LogicOp{
            add("+", (a, b) -> a + b),
            not("flip", a -> ~(long)(a)),
        }
    "#;
    let condition_op = r#"
        public enum ConditionOp{
            equal("==", (a, b) -> true),
            always("always", (a, b) -> true);
        }
    "#;

    let manifest = parse_representative_manifest("158.1", statements, logic_op, condition_op)
        .expect("source parses");

    let names: Vec<_> = manifest.instructions.iter().map(|i| i.rust_name.as_str()).collect();
    assert!(names.contains(&"set"));
    assert!(names.contains(&"op_add"));
    assert!(names.contains(&"op_not"));
    assert!(names.contains(&"jump_equal"));
    assert!(names.contains(&"jump_always"));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test -p mlcg_builtin_gen --test source_parser
```

Expected: FAIL because `source_parser` is empty.

- [ ] **Step 3: Implement representative parser**

Implement:

```rust
pub fn parse_representative_manifest(
    version: &str,
    l_statements: &str,
    logic_op: &str,
    condition_op: &str,
) -> Result<Manifest, GenerateError>
```

Rules:
- require `@RegisterStatement("set")` in `l_statements`;
- require `add(` in `LogicOp.java` and classify as binary;
- require `not(` in `LogicOp.java` and classify as unary;
- require `equal(` and `always(` in `ConditionOp.java`;
- produce instructions matching the current committed manifest shape.

- [ ] **Step 4: Verify and commit**

```bash
cargo fmt --all
cargo test -p mlcg_builtin_gen --test source_parser
cargo test -p mlcg_builtin_gen --test fixture_parser
git add crates/mlcg_builtin_gen
git commit -m "feat(gen): parse representative Mindustry source"
```

## Task 3: Add git cache management

**Files:**
- Create/modify: `crates/mlcg_builtin_gen/src/cache.rs`
- Test: `crates/mlcg_builtin_gen/tests/cache.rs`

- [ ] **Step 1: Write cache path/validation tests**

Create tests for:
- `default_cache_path("158.1")` ends with `target/mlcg-cache/mindustry/v158.1`;
- `validate_mindustry_cache(path)` returns an error listing missing `LStatements.java` when the path is empty.

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p mlcg_builtin_gen --test cache
```

Expected: FAIL because cache functions are not implemented.

- [ ] **Step 3: Implement cache module**

Implement:

```rust
pub fn default_cache_path(version: &str) -> PathBuf;
pub fn validate_mindustry_cache(path: &Path) -> Result<(), GenerateError>;
pub fn ensure_mindustry_cache(version: &str) -> Result<PathBuf, GenerateError>;
```

`ensure_mindustry_cache`:
- builds default path;
- if path does not exist, runs `git clone --depth 1 --branch v{version} https://github.com/Anuken/Mindustry.git <path>`;
- validates expected files exist.

- [ ] **Step 4: Verify and commit**

```bash
cargo fmt --all
cargo test -p mlcg_builtin_gen --test cache
git add crates/mlcg_builtin_gen
git commit -m "feat(gen): add Mindustry git cache"
```

## Task 4: Wire CLI fetch command and real source reading

**Files:**
- Modify: `crates/mlcg_builtin_gen/src/main.rs`
- Modify: `crates/mlcg_builtin_gen/src/source_parser.rs`

- [ ] **Step 1: Add library helper**

Add:

```rust
pub fn parse_cached_mindustry(version: &str, cache_path: &Path) -> Result<Manifest, GenerateError>
```

It reads the three real source files and calls `parse_representative_manifest`.

- [ ] **Step 2: Update CLI**

Support:

```bash
mlcg_builtin_gen fetch 158.1 crates/mlcg_builtin/manifests/v158_1.toml
mlcg_builtin_gen fixture 158.1 input.txt output.toml
```

Keep old positional fixture behavior only if easy; otherwise replace it with explicit subcommands.

- [ ] **Step 3: Verify CLI help/error path**

Run:

```bash
cargo run -p mlcg_builtin_gen --
```

Expected: prints usage and exits non-zero or panics with a clear usage message.

- [ ] **Step 4: Run offline tests and commit**

```bash
cargo fmt --all
cargo test -p mlcg_builtin_gen --all-targets
git add crates/mlcg_builtin_gen
git commit -m "feat(gen): wire fetch command"
```

## Task 5: Manual real fetch verification and final checks

**Files:**
- No required source changes unless real fetch exposes a parser bug.

- [ ] **Step 1: Run real fetch command**

Run:

```bash
cargo run -p mlcg_builtin_gen -- fetch 158.1 /tmp/mlcg-v158_1.toml
```

Expected: command succeeds and `/tmp/mlcg-v158_1.toml` contains `set`, `op_add`, `op_not`, `jump_equal`, and `jump_always`.

- [ ] **Step 2: Compare generated representative manifest**

Run:

```bash
grep -E 'rust_name = "(set|op_add|op_not|jump_equal|jump_always)"' /tmp/mlcg-v158_1.toml
```

Expected: all five names are present.

- [ ] **Step 3: Run full verification**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

Expected: all commands exit 0.

- [ ] **Step 4: Commit any final parser fixes**

If Step 1 required source changes:

```bash
git add crates/mlcg_builtin_gen
git commit -m "fix(gen): handle v158_1 source layout"
```

If no source changes were required, do not create an empty commit.
