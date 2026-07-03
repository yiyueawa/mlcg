# Raw Statement Scan Design

Date: 2026-07-03

## Purpose

Add a raw/source manifest layer for Mindustry logic statements. This layer scans every `@RegisterStatement` class in `LStatements.java` and records source facts without claiming that they are final semantic Rust APIs.

The raw scan is a foundation for later semantic inference. It should not replace the current generated semantic manifest used by `mlcg_builtin`.

## Command

Add:

```bash
cargo run -p mlcg_builtin_gen -- scan-statements 158.1 /tmp/mlcg-statements-v158_1.toml
```

The command reuses the existing Mindustry git cache and reads:

```text
core/src/mindustry/logic/LStatements.java
```

## Raw manifest shape

The raw scan writes TOML with a separate shape from the current semantic instruction manifest:

```toml
version = "158.1"

[[statements]]
name = "read"
class = "ReadStatement"
instruction = "ReadI"
category = "io"
fields = [
  { type = "String", name = "output", default = "result" },
  { type = "String", name = "target", default = "cell1" },
  { type = "String", name = "address", default = "0" },
]
```

Fields are source fields from the statement class. The first implementation only needs public instance fields declared directly in the class body. It should skip `transient` and `static` fields.

## Parser strategy

Use a targeted brace-aware scanner rather than full Java parsing.

1. Find `@RegisterStatement("...")` occurrences.
2. Find the following `public static class <ClassName> extends LStatement` declaration.
3. Extract the class body by matching braces.
4. Inside the body:
   - parse public field declarations with simple defaults;
   - find `return new XxxI(...)` inside `build(LAssembler builder)`;
   - find `return LCategory.xxx` inside `category()`.

The parser should tolerate complex field declarations by extracting what it can and leaving missing fields empty, but required statement entries should still be present.

## Tests

Normal tests remain offline:

- a fixture with `read`, `set`, `op`, and `jump` classes;
- assertions for name, class, fields, instruction, category;
- ensure transient/static fields are skipped.

Real verification can run the command against cached/fetched v158.1 and assert it sees at least 50 statements and known statements.

## Non-goals

- Generating user-facing Rust API from raw scan output.
- Full Java parsing.
- Inferring receiver/input/output semantic roles.
- Parsing enum variant semantics.
- Replacing `crates/mlcg_builtin/manifests/v158_1.toml`.
