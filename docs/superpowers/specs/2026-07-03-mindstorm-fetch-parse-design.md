# Mindustry v158.1 Fetch and Representative Parse Design

Date: 2026-07-03

## Purpose

Extend `mlcg_builtin_gen` from fixture parsing to a real-source vertical slice. The generator should fetch Mindustry v158.1 into a local cache, parse representative real source files, and emit a manifest compatible with `mlcg_builtin_macros`.

This phase is intentionally not the full instruction-table parser. It proves the real-source path and covers representative semantics needed by the current generated API.

## Scope

The generator will support a command shape like:

```bash
cargo run -p mlcg_builtin_gen -- fetch 158.1 crates/mlcg_builtin/manifests/v158_1.toml
```

It will:

1. Ensure Mindustry `v158.1` is available in a local cache.
2. Read real files from the cache:
   - `core/src/mindustry/logic/LStatements.java`
   - `core/src/mindustry/logic/LogicOp.java`
   - `core/src/mindustry/logic/ConditionOp.java`
3. Generate a TOML manifest using the existing representative schema.
4. Cover:
   - `set`
   - `op add`
   - `op not`
   - `jump always`
   - `jump equal`

If parsing discovers more op/jump variants cheaply, the parser may include them, but tests for this phase should assert only the representative set above.

## Cache strategy

Use the system `git` executable instead of `git2` or archive downloads.

Default cache root:

```text
target/mlcg-cache/mindustry/v158.1/
```

Behavior:

- If the cache path does not exist, run:
  ```bash
  git clone --depth 1 --branch v158.1 https://github.com/Anuken/Mindustry.git <cache-path>
  ```
- If the cache path already exists, reuse it after validating that expected source files exist.
- If expected files are missing, return a semantic error telling the user which cached path is invalid.

The first implementation does not need cache invalidation, forced refresh, or commit-hash verification.

## Parser strategy

Use targeted text parsing for the representative source facts, not a full Java parser.

### `set`

Detect `@RegisterStatement("set")` in `LStatements.java` and generate the existing `set` entry:

```toml
family = "set"
variant = "set"
rust_name = "set"
emit = ["set", "$target", "$source"]
receiver = "target"
inputs = ["source"]
outputs = []
```

This entry is stable enough for this phase because `SetStatement` maps directly to `SetI(builder.var(from), builder.var(to))`, while textual writing emits statement fields in registered field order.

### `LogicOp`

Parse enum entries from `LogicOp.java`.

For this phase, identify at least:

- `add`: binary, emits `op add $out $lhs $rhs`
- `not`: unary, emits `op not $out $input 0`

The parser can classify unary entries by detecting constructor calls with one lambda argument shape, and binary entries by two-argument lambda shape or known entry forms. If classification is uncertain, skip that variant unless it is one of the required representative variants, in which case return an error.

### `ConditionOp`

Parse enum entries from `ConditionOp.java`.

For this phase, identify at least:

- `always`: emits `jump $label always 0 0`, no value inputs
- `equal`: emits `jump $label equal $lhs $rhs`, two value inputs

The parser should use enum variant names for emitted condition tokens. This matches the current manifest schema; later phases can add symbol/name metadata if needed.

## Error handling

Use SNAFU errors in `mlcg_builtin_gen`:

- failed git command, including command and status;
- missing cache file;
- failed source read;
- required representative item not found;
- TOML serialization failure.

Messages should be lower-case fragments and should not silently fall back to fixture data.

## Tests

Keep tests deterministic and offline where possible.

1. Unit tests for parser functions with small source snippets for `LStatements`, `LogicOp`, and `ConditionOp`.
2. A CLI/library integration test may use a temporary fake cache path and skip network by calling the parse function directly.
3. Do not make normal `cargo test` depend on GitHub network access.
4. Manual verification may run the real fetch command when network is available.

## Non-goals

- Full `@RegisterStatement` instruction coverage.
- Full Java AST parsing.
- Cache refresh/pinning beyond tag checkout.
- Replacing the committed v158.1 manifest automatically in tests.
- Proc macro changes unless the manifest schema needs a small compatible extension.
