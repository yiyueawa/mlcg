# Raw Manifest to Rust API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Commit the v158.1 raw statement manifest and add a generator path that derives the semantic manifest consumed by `mlcg_builtin_macros` from raw Mindustry statement facts.

**Architecture:** Keep the raw/source manifest under `crates/mlcg_builtin/manifests/raw/` as source facts. Add a focused `semantic_manifest` module in `mlcg_builtin_gen` that converts raw statements into the existing semantic `Manifest` shape, using conservative statement-level rules plus explicit known semantic overrides for `set`, `op.add`, `op.not`, `jump.equal`, and `jump.always`.

**Tech Stack:** Rust, serde/toml, SNAFU, existing `mlcg_builtin_macros::include_manifest!` codegen.

---

## Files

```text
crates/mlcg_builtin/manifests/raw/v158_1_statements.toml     # committed raw v158.1 scan output
crates/mlcg_builtin_gen/src/semantic_manifest.rs             # raw -> semantic conversion
crates/mlcg_builtin_gen/src/lib.rs                           # export semantic_manifest
crates/mlcg_builtin_gen/src/main.rs                          # add derive-semantic command
crates/mlcg_builtin_gen/tests/semantic_manifest.rs           # TDD tests for conversion
crates/mlcg_builtin/manifests/v158_1.toml                    # regenerated semantic manifest
crates/mlcg_builtin/tests/v158_1.rs                          # API coverage for derived statement methods
```

## Task 1: Commit raw v158.1 statement manifest

- [ ] **Step 1: Generate manifest**

```bash
mkdir -p crates/mlcg_builtin/manifests/raw
cargo run -p mlcg_builtin_gen -- scan-statements 158.1 crates/mlcg_builtin/manifests/raw/v158_1_statements.toml
```

- [ ] **Step 2: Validate manifest shape**

```bash
grep -c '^\[\[statements\]\]' crates/mlcg_builtin/manifests/raw/v158_1_statements.toml
grep -E 'name = "(read|set|op|jump|control|ucontrol)"' crates/mlcg_builtin/manifests/raw/v158_1_statements.toml
```

Expected: count is 52 and known statement names are present.

## Task 2: Add raw-to-semantic conversion with tests

- [ ] **Step 1: Write failing tests**

Create `crates/mlcg_builtin_gen/tests/semantic_manifest.rs` with fixtures that verify:

```rust
use mlcg_builtin_gen::{raw_statement::scan_raw_statements, semantic_manifest::derive_semantic_manifest};

#[test]
fn derives_known_semantic_overrides_and_generic_statement_apis() {
    let source = r#"... read, set, print, op, jump fixture ..."#;
    let raw = scan_raw_statements("158.1", source).expect("scan succeeds");
    let manifest = derive_semantic_manifest(&raw);

    assert!(manifest.instructions.iter().any(|i| i.rust_name == "set" && i.receiver == "target"));
    assert!(manifest.instructions.iter().any(|i| i.rust_name == "op_add" && i.outputs == ["out"]));
    assert!(manifest.instructions.iter().any(|i| i.rust_name == "jump_equal"));
    assert!(manifest.instructions.iter().any(|i| i.rust_name == "print" && i.emit == ["print", "$value"]));
}
```

- [ ] **Step 2: Run red test**

```bash
cargo test -p mlcg_builtin_gen --test semantic_manifest
```

Expected: FAIL because `semantic_manifest` does not exist.

- [ ] **Step 3: Implement converter**

Add `derive_semantic_manifest(raw: &RawStatementManifest) -> Manifest` with:

- explicit `set`, `op_add`, `op_not`, `jump_equal`, `jump_always` entries;
- one generic instruction for each raw statement that is not superseded by explicit overrides;
- generic emit tokens as `[statement.name, "$field1", "$field2", ...]`;
- generic inputs as all placeholder fields, no receiver, no auto outputs.

- [ ] **Step 4: Run green test**

```bash
cargo test -p mlcg_builtin_gen --test semantic_manifest
```

Expected: PASS.

## Task 3: Wire CLI, regenerate semantic manifest, and test Rust API

- [ ] **Step 1: Add CLI command**

Add:

```bash
mlcg_builtin_gen derive-semantic <raw-input-toml> <semantic-output-toml>
```

It reads `RawStatementManifest` from TOML, calls `derive_semantic_manifest`, and writes pretty TOML.

- [ ] **Step 2: Regenerate semantic manifest**

```bash
cargo run -p mlcg_builtin_gen -- derive-semantic crates/mlcg_builtin/manifests/raw/v158_1_statements.toml crates/mlcg_builtin/manifests/v158_1.toml
```

- [ ] **Step 3: Add API test**

Extend `crates/mlcg_builtin/tests/v158_1.rs` to call a generic generated method such as:

```rust
processor.print("message");
processor.read_into(x.clone(), "cell1", 0);
```

Assert emitted mlog includes `print message` and `read x cell1 0`.

- [ ] **Step 4: Run focused tests**

```bash
cargo test -p mlcg_builtin_gen --test semantic_manifest
cargo test -p mlcg_builtin --test v158_1
```

Expected: PASS.

## Task 4: Full verification, commit, push

- [ ] **Step 1: Format, lint, test**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

Expected: all exit 0.

- [ ] **Step 2: Commit only task files**

```bash
git add docs/superpowers/plans/2026-07-03-raw-manifest-to-rust.md crates/mlcg_builtin/manifests/raw/v158_1_statements.toml crates/mlcg_builtin/manifests/v158_1.toml crates/mlcg_builtin_gen/src crates/mlcg_builtin_gen/tests crates/mlcg_builtin/tests/v158_1.rs
git commit -m "feat(gen): derive builtin api from raw manifest"
```

- [ ] **Step 3: Push as yiyueawa**

```bash
gh auth switch -u yiyueawa
git push
```
