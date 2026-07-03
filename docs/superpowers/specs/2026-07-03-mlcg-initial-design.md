# mlcg Initial Design

Date: 2026-07-03

## Purpose

`mlcg` is the Mindustry Logic Code Generator. Users write ordinary Rust code that constructs Mindustry Logic (`mlog`) programs at runtime. The project is not a compiler for a new language. Rust is the host language; running Rust code emits mlog text.

The first usable version is library-first. It prioritizes a clean core model, a generated builtin layer, and a reproducible manifest pipeline for Mindustry v158.1.

## Workspace layout

The workspace contains five crates:

```text
mlcg/
  Cargo.toml
  crates/
    mlcg_core/
    mlcg_builtin/
    mlcg_builtin_macros/
    mlcg_builtin_gen/
    mlcg/
```

### `mlcg_core`

`mlcg_core` defines only the infrastructure that is independent of Mindustry's concrete instruction table:

- `Processor<P>`: processor/program construction context. `P` is an explicit processor marker.
- `ProcessorHandle<P>`: shared, thread-safe handle stored by processors and values.
- `Value<P, T = Any>`: value identity bound to one processor. `T` is a marker stored as `_type: PhantomData<T>`; the first version uses `Any` everywhere.
- `Any`: default type marker.
- `Label<P>`: label identity bound to one processor.
- `Instruction<P>`: trait implemented by generated or hand-written instructions.
- Lowering and emission contexts.
- Partial program representation used between instruction lowering and final text emission.

`mlcg_core` does not define Mindustry-specific concepts such as `Operand`, `Expr`, `Set`, `Op`, `JumpCondition`, `LAccess`, or concrete instruction enums. Those belong to generated builtin code.

### `mlcg_builtin`

`mlcg_builtin` exposes Mindustry builtin APIs generated from committed manifests:

- `mlcg_builtin::v158_1` for Mindustry v158.1.
- `mlcg_builtin::latest` as a re-export of the latest supported version.
- A version prelude that re-exports generated extension traits and enums.

It may also contain hand-written sugar extensions in separate modules, but the first version should not blur those with generated APIs.

### `mlcg_builtin_macros`

`mlcg_builtin_macros` is a proc-macro crate. It reads committed manifest files and generates Rust instruction structs, argument types, enums, processor-level extension traits, and value-level extension traits.

The macro crate does not parse Mindustry source code and does not access the network.

### `mlcg_builtin_gen`

`mlcg_builtin_gen` is a normal development tool. It reads Mindustry source code for a specific version and writes manifest files. It is the only component that understands Mindustry's Java source layout.

The first target version is v158.1. Future versions can add version modules and re-export unchanged symbols from previous versions.

### `mlcg`

`mlcg` is the user-facing facade crate. It re-exports common `mlcg_core` types, the latest builtin prelude, and helper macros such as `processor!()`.

## Processor identity and values

Users create processors with a macro rather than hand-writing marker types:

```rust
use mlcg::prelude::*;

let processor = processor!();
let x = processor.named("x");
let tmp = processor.new();
```

`processor!()` expands to a fresh processor marker type and returns `Processor<GeneratedMarker>`. Values created by different processor markers are different Rust types and cannot be directly mixed by generated APIs. This leaves room for future explicit cross-processor operations.

`Value<P, T = Any>` contains:

- a value id;
- a `ProcessorHandle<P>`;
- a delayed name hint;
- `_type: PhantomData<T>`.

The first version uses `Any` for all generated APIs. More specific marker types such as `Number`, `Bool`, or `Text` can be added later without making the core model depend on them now.

Values carry their processor handle so receiver-style extension methods can push instructions directly:

```rust
let x = processor.named("x");
x.set(1);
```

## Threading model

The first version should support multi-threaded construction. `ProcessorHandle<P>` is thread-safe, conceptually an `Arc<Mutex<ProgramState<P>>>` or an equivalent synchronization primitive.

Goals:

- `Processor<P>` and `Value<P, T>` can be cloned and sent to other threads when their marker permits it.
- Concurrent instruction pushes do not cause data races.
- Each pushed item receives a deterministic sequence inside the locked state.

The first version only guarantees thread safety. It does not promise that unsynchronized concurrent pushes produce a stable semantic order across runs. Users who need stable ordering must synchronize their own construction flow; future APIs may offer ordered blocks or mergeable builders.

## Labels

Labels belong to `mlcg_core` because they are not a concrete Mindustry instruction. They are positioning markers used by instructions such as `jump`.

Labels are two-phase:

```rust
let end = processor.label(); // may be referenced before placement
processor.jump_always(end);
processor.place(end);        // establishes final location
```

A label can be created before it is placed. During emission, unplaced labels are errors.

## Instruction lowering and final emission

`Instruction<P>` does not write final mlog text directly. An instruction may lower to zero, one, or many mlog lines. The core emission pipeline is therefore two-phase.

### Phase 1: lower original program items

The original program stream contains instruction objects and label placements:

```rust
enum ProgramItem<P> {
    Instruction(Box<dyn Instruction<P>>),
    LabelPlacement(Label<P>),
}
```

Each instruction implements a trait conceptually like:

```rust
pub trait Instruction<P>: Send + Sync + 'static {
    fn lower(
        &self,
        ctx: &mut LowerContext<P>,
        out: &mut PartialProgram<P>,
    ) -> Result<(), LowerError>;
}
```

`PartialProgram<P>` contains partially resolved lines and tokens. Tokens may reference values or labels that are resolved later.

When a `LabelPlacement` is seen, core records the current partial line index. If the previous instruction lowered to multiple lines, the label naturally points to the line after that expansion.

### Phase 2: resolve and write mlog text

After all instructions lower:

- value references become final mlog variable names;
- label references become final numeric line numbers;
- literal/raw/symbol tokens become textual tokens;
- each partial line becomes one final mlog line.

This design lets generated one-line instructions, future pseudo-instructions, and hand-written sugar share the same label resolution mechanism.

## Delayed value naming

Values have stable internal ids and optional name hints. Final mlog variable names are assigned during emission.

This allows the emitter to:

- preserve user-readable names where possible;
- avoid collisions;
- rename temporaries;
- later support compact/minified naming.

## Core/builtin boundary

`mlcg_core` owns only generic construction, value identity, labels, lowering, and text emission. `mlcg_builtin` owns all concrete Mindustry semantics generated from manifests.

Generated builtin code implements `Instruction<P>` for concrete instruction structs. For example, `Set<P>` and `OpAdd<P>` are builtin-generated structs, not variants of a core instruction enum.

`set`, `op`, `jump`, `control`, `sensor`, and similar APIs are extension traits generated by `mlcg_builtin_macros` from manifests. They are not core methods, except for generic helpers such as `push`, `new`, `named`, `label`, `place`, and `emit`.

## Manifest role

Manifest files are committed to the repository and are the stable interface between source analysis and Rust code generation.

```text
Mindustry v158.1 source
  -> mlcg_builtin_gen
  -> committed TOML manifest
  -> mlcg_builtin_macros
  -> mlcg_builtin::v158_1
  -> mlcg_core lowering and emission
```

TOML is the chosen manifest format. The exact TOML layout is intentionally not fixed in this design because it depends on how the generator interprets Mindustry source code. The schema must nevertheless be able to express the concepts below.

## Instruction families, variants, and arity

The generator must distinguish instruction families from semantic variants.

For example, Mindustry source represents operations as an `op` statement family plus `LogicOp` variants. The Rust API should understand `op add` and `op not` as different generated operation APIs:

```rust
let z = processor.op_add(x, y);
let n = processor.op_not(x);
processor.op_add_into(out, x, y);
processor.op_not_into(out, x);
```

Similarly, `jump always` and `jump equal` should produce different semantic APIs:

```rust
processor.jump_always(label);
processor.jump_equal(label, x, y);
```

The manifest must separate:

- statement family, such as `op`, `jump`, `control`, or `ucontrol`;
- variant, such as `add`, `not`, `always`, `equal`, or a control enum member;
- semantic inputs and outputs exposed to Rust users;
- emit layout required by Mindustry mlog text;
- default or unused emitted fields.

This separation is necessary because semantic arity and emitted field count may differ. For example, `op not` has one semantic input and one output, while the textual representation may still use a fixed operation layout with an unused/default field.

## Mindustry source facts guiding the first generator

The first generator targets Mindustry v158.1. Initial source review shows:

- `LStatements.OperationStatement` uses a statement family named `op` and a `LogicOp` enum.
- `LogicOp` contains metadata such as `unary` and `func`; `not` is unary, while `add` is binary.
- `JumpStatement` uses `ConditionOp`; `always` is semantically different from comparisons that need two operands.
- `LogicStatementProcessor` generates read/write logic for registered statements by writing registered statement names and fields.
- Reads tolerate missing trailing fields by keeping statement defaults, so the project should not assume every shorter line is invalid.
- `LParser` supports labels and later resolves label-like `jump` destinations to line numbers.

The generator should use these source structures rather than relying on fragile line-format guesses. If a required default value, field role, or variant role cannot be extracted confidently, generation should fail or mark the manifest entry as requiring manual confirmation. It should not silently invent semantics.

## Generated API layers

Generated APIs come in two main forms.

### Processor-level extension traits

Processor-level APIs accept inputs and explicit outputs, or allocate a single output when the manifest says that is valid:

```rust
processor.set(x, 1);
processor.op_add_into(out, x, y);
let z = processor.op_add(x, y);
processor.jump_equal(label, x, y);
```

### Value-level receiver extension traits

Value-level APIs are generated when the manifest clearly identifies a receiver/subject parameter:

```rust
x.set(1);
```

The receiver is not guessed from syntax alone. It is derived from manifest semantics.

### Hand-written sugar

More idiomatic Rust helpers, such as `x.add_assign(y)`, are outside automatic generation for the first version. They can be added later as hand-written sugar modules layered above generated APIs.

## Operator overloading

The first version does not implement Rust standard operator overloading such as `+`, `-`, `*`, or `==`.

This avoids forcing builtin semantics back into `mlcg_core`. Standard operator overloading can be revisited after the core/builtin boundary and generated API shape are stable.

## Version modules

The first supported version is v158.1:

```rust
pub mod v158_1;
pub mod latest {
    pub use super::v158_1::*;
}
```

Future versions may re-export unchanged symbols from previous versions and define only changed symbols locally:

```rust
pub mod v159_0 {
    pub use super::v158_1::{unchanged_a, unchanged_b};
    // changed or new definitions generated here
}
```

The manifest design should leave room for version inheritance/re-export metadata, but the first implementation only needs v158.1.

## Testing strategy

### Core tests

`mlcg_core` tests use fake instructions and do not depend on Mindustry source:

- create processors and values;
- prove values carry handles and can push receiver-style instructions through extension traits;
- lower one instruction to multiple partial lines;
- place labels before and after multi-line lowering;
- resolve labels to final line numbers;
- return errors for unplaced labels;
- verify delayed value naming and collision handling;
- verify multi-threaded pushes do not lose items;
- use compile-fail tests to show different processor markers cannot be mixed.

### Builtin generator tests

`mlcg_builtin_gen` tests should start with small fixture sources shaped like the relevant Mindustry classes. They should verify extraction of:

- statement family names;
- variants from enum-like sources;
- unary vs binary operation variants;
- semantic inputs and outputs;
- emit layout and defaults.

Real v158.1 source support should be added after fixture behavior is stable.

### Macro tests

`mlcg_builtin_macros` tests use small manifest fixtures and verify generated code for:

- instruction structs;
- generated enums;
- processor-level extension traits;
- value-level receiver extension traits;
- lowering to partial lines;
- default/unused emitted fields.

### End-to-end tests

At least one example should use `mlcg::prelude::*`, generated builtin APIs, labels, and final emission. Representative coverage should include:

- `set`;
- `op add`;
- `op not`;
- `jump always`;
- `jump equal`;
- one more complex family such as `control` or `ucontrol` if extraction is ready.

## First implementation boundary

The first implementation should deliver a working vertical slice:

1. workspace and crate scaffolding;
2. `mlcg_core` processor/value/label/lower/emit infrastructure;
3. a small TOML manifest schema sufficient for representative instruction families;
4. fixture-driven `mlcg_builtin_gen` extraction;
5. proc macro generation from the fixture/committed manifest;
6. `mlcg_builtin::v158_1` and `latest` modules;
7. facade `mlcg` prelude and `processor!()` macro;
8. an end-to-end example that emits mlog.

The long-term target remains complete Mindustry v158.1 instruction coverage. If full v158.1 extraction requires more research than the first slice can absorb, the first slice may cover representative families while keeping the schema expressive enough for the full table.

## Non-goals for the first version

- No high-level `if_` or `while_` DSL.
- No Rust standard operator overloading.
- No complete static value type system beyond `Any`.
- No implicit cross-processor operation.
- No proc macro network access or source parsing.
- No silent handwritten replacement for missing generator knowledge.
