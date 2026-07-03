# Semantic Rule Generation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace instruction-name special cases with parameter-driven semantic derivation and rename generated extension traits to `Processor*Ext` / `Value*Ext`.

**Architecture:** Extend the raw/source manifest with enum variant metadata. Derive semantic instructions using general rules: enum fields expand variants into Rust symbols and literal emit tokens, output-like fields become outputs, and the first remaining operand-like field becomes the value receiver. The proc macro keeps the existing semantic manifest shape but derives trait names with the receiver kind prefix.

**Tech Stack:** Rust, serde/toml, SNAFU, proc-macro2/quote, cargo tests.

---

## Files

```text
crates/mlcg_builtin_gen/src/raw_statement.rs             # add RawEnum + enum parsing helper
crates/mlcg_builtin_gen/src/main.rs                      # scan enum source files into raw manifest
crates/mlcg_builtin_gen/src/semantic_manifest.rs         # remove name special cases; add rule-based derivation
crates/mlcg_builtin_gen/tests/semantic_manifest.rs       # red/green unit tests for rule generation
crates/mlcg_builtin_gen/tests/raw_statement.rs           # raw enum parse tests
crates/mlcg_builtin_macros/src/generate.rs               # rename traits Processor*Ext / Value*Ext
crates/mlcg_builtin_macros/tests/macro_basic.rs          # trait name compile test
crates/mlcg_builtin_macros/tests/ui/pass_macro_basic.rs  # imported trait names
crates/mlcg_builtin/manifests/raw/v158_1_statements.toml # regenerated raw manifest with enum metadata
crates/mlcg_builtin/manifests/v158_1.toml                # regenerated semantic manifest
crates/mlcg_builtin/tests/v158_1.rs                      # API test for value receiver methods
```

## Rule Summary

- Enum fields: if a raw field type has known variants, create one semantic instruction per enum variant.
- Emit order: statement name, enum variant literals in field order, then non-enum operand placeholders in field order.
- Output fields: `output`, `result`, `dest`, and `out[A-Z]*`.
- Receiver field: first non-output operand field, when one exists.
- Processor auto output: one output field returns `Value<P>` and also has `_into`.
- Value receiver: generated whenever a receiver field exists; method uses remaining input fields and returns `Value<P>` when exactly one output exists.
- No branch in semantic derivation may match concrete statement names such as `set`, `op`, or `jump`.

## Verification

Run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```
