# Raw Statement Scan Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `mlcg_builtin_gen scan-statements` to emit a raw TOML manifest of all `@RegisterStatement` classes from Mindustry `LStatements.java`.

**Architecture:** Add a raw statement model and brace-aware source scanner under `mlcg_builtin_gen`. Keep raw scan output separate from the semantic manifest consumed by `mlcg_builtin_macros`.

**Tech Stack:** Rust, SNAFU, serde/toml, existing git cache module.

---

## Files

```text
crates/mlcg_builtin_gen/src/raw_statement.rs       # raw model + scanner
crates/mlcg_builtin_gen/src/lib.rs                 # export module
crates/mlcg_builtin_gen/src/main.rs                # scan-statements command
crates/mlcg_builtin_gen/tests/raw_statement.rs     # offline scanner tests
```

## Task 1: Add raw statement scanner with offline tests

- [ ] **Step 1: Write failing tests**

Create `crates/mlcg_builtin_gen/tests/raw_statement.rs` with a fixture containing `read`, `set`, `op`, and `jump`. Assert parsed statement count, class names, fields, instruction classes, categories, and skipped transient/static fields.

- [ ] **Step 2: Run failing test**

```bash
cargo test -p mlcg_builtin_gen --test raw_statement
```

Expected: FAIL because module does not exist.

- [ ] **Step 3: Implement model and scanner**

Create `src/raw_statement.rs` with:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct RawStatementManifest { pub version: String, pub statements: Vec<RawStatement> }
#[derive(Debug, Clone, serde::Serialize)]
pub struct RawStatement { pub name: String, pub class: String, pub instruction: Option<String>, pub category: Option<String>, pub fields: Vec<RawField> }
#[derive(Debug, Clone, serde::Serialize)]
pub struct RawField { pub ty: String, pub name: String, pub default: Option<String> }

pub fn scan_raw_statements(version: &str, source: &str) -> Result<RawStatementManifest, GenerateError>
```

Implement a brace-aware class body extractor and simple public field parser.

- [ ] **Step 4: Verify and commit**

```bash
cargo fmt --all
cargo test -p mlcg_builtin_gen --test raw_statement
git add crates/mlcg_builtin_gen
git commit -m "feat(gen): scan raw logic statements"
```

## Task 2: Wire scan-statements CLI

- [ ] **Step 1: Update lib exports**

Export `raw_statement` from `src/lib.rs`.

- [ ] **Step 2: Update main**

Add subcommand:

```bash
mlcg_builtin_gen scan-statements <version> <output-toml>
```

It uses `ensure_mindustry_cache`, reads `LStatements.java`, calls `scan_raw_statements`, and writes `toml::to_string_pretty` output.

- [ ] **Step 3: Run real command**

```bash
cargo run -p mlcg_builtin_gen -- scan-statements 158.1 /tmp/mlcg-statements-v158_1.toml
```

Expected: succeeds.

- [ ] **Step 4: Validate real output**

```bash
grep -c '^\[\[statements\]\]' /tmp/mlcg-statements-v158_1.toml
grep -E 'name = "(read|set|op|jump|control|ucontrol)"' /tmp/mlcg-statements-v158_1.toml
```

Expected: count is at least 50 and known names appear.

- [ ] **Step 5: Full verification and commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
git add crates/mlcg_builtin_gen
git commit -m "feat(gen): add raw statement scan command"
```
