# mlcg Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working `mlcg` vertical slice: core processor/value/label/lowering infrastructure, generated builtin APIs from a manifest, a fixture source generator, a facade crate, and an end-to-end mlog emission example.

**Architecture:** `mlcg_core` owns processor identity, values, labels, thread-safe program state, partial-line lowering, and final text emission. `mlcg_builtin_macros` generates concrete instruction structs and extension traits from committed TOML manifests; `mlcg_builtin` exposes version modules; `mlcg_builtin_gen` produces manifests from fixture Mindustry-like source; `mlcg` re-exports a prelude and `processor!()`.

**Tech Stack:** Rust workspace, Cargo, SNAFU for semantic errors, `serde`/`toml` for manifests, `syn`/`quote` for proc macros, `trybuild` for compile-fail tests.

---

## File structure

Create or modify these files:

```text
Cargo.toml                                      # workspace members and shared package settings
crates/mlcg_core/Cargo.toml                    # core crate manifest
crates/mlcg_core/src/lib.rs                    # public core facade
crates/mlcg_core/src/error.rs                  # core error types
crates/mlcg_core/src/id.rs                     # ValueId and LabelId newtypes
crates/mlcg_core/src/processor.rs              # Processor, handle, program state
crates/mlcg_core/src/value.rs                  # Value<P, T>, Any marker
crates/mlcg_core/src/label.rs                  # Label<P>
crates/mlcg_core/src/instruction.rs            # Instruction trait and ProgramItem
crates/mlcg_core/src/lower.rs                  # partial program, tokens, lower context
crates/mlcg_core/src/emit.rs                   # final emitter and name allocation
crates/mlcg_core/tests/core_emit.rs            # behavior tests for lowering, labels, names
crates/mlcg_core/tests/threading.rs            # thread-safety behavior test

crates/mlcg_builtin_macros/Cargo.toml          # proc-macro manifest
crates/mlcg_builtin_macros/src/lib.rs          # include_manifest! macro entrypoint
crates/mlcg_builtin_macros/src/manifest.rs     # manifest data model used by macro
crates/mlcg_builtin_macros/src/generate.rs     # Rust token generation
crates/mlcg_builtin_macros/tests/macro_basic.rs# macro integration test

crates/mlcg_builtin/Cargo.toml                 # builtin facade manifest
crates/mlcg_builtin/src/lib.rs                 # version modules and latest
crates/mlcg_builtin/src/v158_1.rs              # invokes macro on committed manifest
crates/mlcg_builtin/manifests/v158_1.toml      # representative committed manifest
crates/mlcg_builtin/tests/v158_1.rs            # generated API behavior tests

crates/mlcg_builtin_gen/Cargo.toml             # generator CLI manifest
crates/mlcg_builtin_gen/src/main.rs            # CLI entrypoint
crates/mlcg_builtin_gen/src/lib.rs             # generator library exports
crates/mlcg_builtin_gen/src/fixture_parser.rs  # fixture source extraction
crates/mlcg_builtin_gen/tests/fixtures/v158_1_logic.txt # small fixture source
crates/mlcg_builtin_gen/tests/fixture_parser.rs# generator behavior tests

crates/mlcg/Cargo.toml                         # facade crate manifest
crates/mlcg/src/lib.rs                         # public prelude and processor! macro
crates/mlcg/tests/processor_macro.rs           # compile/runtime macro tests
crates/mlcg/tests/ui/mixed_processors.rs       # trybuild compile-fail source
crates/mlcg/tests/ui/pass_same_processor.rs    # trybuild compile-pass source
examples/basic.rs                              # end-to-end example
```

Do not modify the existing untracked `.gitignore` unless the user explicitly asks.

## Global verification commands

Run these after each task that changes Rust code:

```bash
cargo fmt --all
cargo test --workspace --all-targets
```

Run this before final completion:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

## Task 1: Scaffold the workspace and crates

**Files:**
- Create: `Cargo.toml`
- Create: each crate `Cargo.toml`
- Create: each crate `src/lib.rs` or `src/main.rs`

- [ ] **Step 1: Create workspace manifest**

Create `Cargo.toml`:

```toml
[workspace]
members = [
    "crates/mlcg_core",
    "crates/mlcg_builtin_macros",
    "crates/mlcg_builtin",
    "crates/mlcg_builtin_gen",
    "crates/mlcg",
]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yiyue/mlcg"

[workspace.dependencies]
mlcg_core = { path = "crates/mlcg_core" }
mlcg_builtin = { path = "crates/mlcg_builtin" }
mlcg_builtin_macros = { path = "crates/mlcg_builtin_macros" }
snafu = "0.8"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
syn = { version = "2", features = ["full"] }
quote = "1"
proc-macro2 = "1"
trybuild = "1"
tempfile = "3"
```

- [ ] **Step 2: Create crate manifests**

Create `crates/mlcg_core/Cargo.toml`:

```toml
[package]
name = "mlcg_core"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
snafu.workspace = true
```

Create `crates/mlcg_builtin_macros/Cargo.toml`:

```toml
[package]
name = "mlcg_builtin_macros"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[lib]
proc-macro = true

[dependencies]
proc-macro2.workspace = true
quote.workspace = true
serde.workspace = true
syn.workspace = true
toml.workspace = true

[dev-dependencies]
mlcg_core.workspace = true
tempfile.workspace = true
```

Create `crates/mlcg_builtin/Cargo.toml`:

```toml
[package]
name = "mlcg_builtin"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
mlcg_core.workspace = true
mlcg_builtin_macros.workspace = true
```

Create `crates/mlcg_builtin_gen/Cargo.toml`:

```toml
[package]
name = "mlcg_builtin_gen"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
serde.workspace = true
snafu.workspace = true
toml.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

Create `crates/mlcg/Cargo.toml`:

```toml
[package]
name = "mlcg"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
mlcg_core.workspace = true
mlcg_builtin.workspace = true

[dev-dependencies]
trybuild.workspace = true
```

- [ ] **Step 3: Create minimal crate roots**

Create `crates/mlcg_core/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub fn crate_ready() -> bool {
    true
}
```

Create `crates/mlcg_builtin_macros/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

use proc_macro::TokenStream;

#[proc_macro]
pub fn include_manifest(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
```

Create `crates/mlcg_builtin/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod v158_1 {}

pub mod latest {
    pub use super::v158_1::*;
}
```

Create `crates/mlcg_builtin_gen/src/main.rs`:

```rust
fn main() {
    eprintln!("mlcg_builtin_gen is not wired yet");
}
```

Create `crates/mlcg_builtin_gen/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub fn crate_ready() -> bool {
    true
}
```

Create `crates/mlcg/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod prelude {
    pub use mlcg_builtin::latest::*;
    pub use mlcg_core::*;
}
```

- [ ] **Step 4: Verify scaffold builds**

Run:

```bash
cargo fmt --all
cargo test --workspace --all-targets
```

Expected: all crates compile and the test command exits 0.

- [ ] **Step 5: Commit scaffold**

```bash
git add Cargo.toml crates
git commit -m "chore: scaffold mlcg workspace"
```

## Task 2: Implement core processor, value, and ids with tests first

**Files:**
- Replace: `crates/mlcg_core/src/lib.rs`
- Create: `crates/mlcg_core/src/id.rs`
- Create: `crates/mlcg_core/src/processor.rs`
- Create: `crates/mlcg_core/src/value.rs`
- Test: `crates/mlcg_core/tests/core_emit.rs`

- [ ] **Step 1: Write failing tests for value creation and naming hints**

Create `crates/mlcg_core/tests/core_emit.rs`:

```rust
use mlcg_core::Processor;

struct TestProcessor;

#[test]
fn processor_creates_named_and_temporary_values() {
    let processor = Processor::<TestProcessor>::new();

    let named = processor.named("x");
    let temporary = processor.new_value();

    assert_eq!(named.name_hint().as_deref(), Some("x"));
    assert_eq!(temporary.name_hint(), None);
    assert_ne!(named.id(), temporary.id());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p mlcg_core --test core_emit processor_creates_named_and_temporary_values
```

Expected: FAIL because `Processor` is not defined.

- [ ] **Step 3: Implement ids, values, and processor creation**

Replace `crates/mlcg_core/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod id;
pub mod processor;
pub mod value;

pub use id::{LabelId, ValueId};
pub use processor::{Processor, ProcessorHandle};
pub use value::{Any, Value};
```

Create `crates/mlcg_core/src/id.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ValueId(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LabelId(pub(crate) u64);
```

Create `crates/mlcg_core/src/value.rs`:

```rust
use std::marker::PhantomData;

use crate::{processor::ProcessorHandle, ValueId};

#[derive(Debug, Clone, Copy, Default)]
pub struct Any;

#[derive(Debug)]
pub struct Value<P, T = Any> {
    pub(crate) id: ValueId,
    pub(crate) handle: ProcessorHandle<P>,
    pub(crate) name_hint: Option<String>,
    pub(crate) _type: PhantomData<T>,
}

impl<P, T> Clone for Value<P, T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            handle: self.handle.clone(),
            name_hint: self.name_hint.clone(),
            _type: PhantomData,
        }
    }
}

impl<P, T> Value<P, T> {
    pub fn id(&self) -> ValueId {
        self.id
    }

    pub fn name_hint(&self) -> Option<String> {
        self.name_hint.clone()
    }

    pub fn handle(&self) -> ProcessorHandle<P> {
        self.handle.clone()
    }
}
```

Create `crates/mlcg_core/src/processor.rs`:

```rust
use std::{marker::PhantomData, sync::{Arc, Mutex}};

use crate::{value::{Any, Value}, ValueId};

#[derive(Debug)]
pub(crate) struct ProgramState<P> {
    pub(crate) next_value: u64,
    pub(crate) _processor: PhantomData<P>,
}

#[derive(Debug)]
pub struct ProcessorHandle<P> {
    pub(crate) state: Arc<Mutex<ProgramState<P>>>,
}

impl<P> Clone for ProcessorHandle<P> {
    fn clone(&self) -> Self {
        Self { state: Arc::clone(&self.state) }
    }
}

#[derive(Debug, Clone)]
pub struct Processor<P> {
    handle: ProcessorHandle<P>,
}

impl<P> Default for Processor<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> Processor<P> {
    pub fn new() -> Self {
        Self {
            handle: ProcessorHandle {
                state: Arc::new(Mutex::new(ProgramState {
                    next_value: 0,
                    _processor: PhantomData,
                })),
            },
        }
    }

    pub fn handle(&self) -> ProcessorHandle<P> {
        self.handle.clone()
    }

    pub fn new_value(&self) -> Value<P, Any> {
        self.allocate_value(None)
    }

    pub fn named(&self, name: impl Into<String>) -> Value<P, Any> {
        self.allocate_value(Some(name.into()))
    }

    fn allocate_value(&self, name_hint: Option<String>) -> Value<P, Any> {
        let mut state = self.handle.state.lock().expect("program state mutex poisoned");
        let id = ValueId(state.next_value);
        state.next_value += 1;
        drop(state);
        Value { id, handle: self.handle.clone(), name_hint, _type: PhantomData }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run:

```bash
cargo test -p mlcg_core --test core_emit processor_creates_named_and_temporary_values
```

Expected: PASS.

- [ ] **Step 5: Run workspace tests and commit**

```bash
cargo fmt --all
cargo test --workspace --all-targets
git add crates/mlcg_core
git commit -m "feat(core): add processor values and ids"
```

## Task 3: Add instruction lowering, partial tokens, labels, and emit

**Files:**
- Modify: `crates/mlcg_core/src/lib.rs`
- Modify: `crates/mlcg_core/src/processor.rs`
- Create: `crates/mlcg_core/src/error.rs`
- Create: `crates/mlcg_core/src/label.rs`
- Create: `crates/mlcg_core/src/instruction.rs`
- Create: `crates/mlcg_core/src/lower.rs`
- Create: `crates/mlcg_core/src/emit.rs`
- Modify: `crates/mlcg_core/tests/core_emit.rs`

- [ ] **Step 1: Write failing tests for multi-line lowering and label resolution**

Append to `crates/mlcg_core/tests/core_emit.rs`:

```rust
use mlcg_core::{Instruction, Label, LowerContext, PartialLine, PartialProgram, PartialToken};

#[derive(Debug)]
struct RawLine(&'static [&'static str]);

impl Instruction<TestProcessor> for RawLine {
    fn lower(
        &self,
        _ctx: &mut LowerContext<TestProcessor>,
        out: &mut PartialProgram<TestProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(
            self.0.iter().copied().map(PartialToken::raw).collect(),
        ));
        Ok(())
    }
}

#[derive(Debug)]
struct JumpTo(Label<TestProcessor>);

impl Instruction<TestProcessor> for JumpTo {
    fn lower(
        &self,
        _ctx: &mut LowerContext<TestProcessor>,
        out: &mut PartialProgram<TestProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(vec![
            PartialToken::raw("jump"),
            PartialToken::label(self.0.clone()),
            PartialToken::raw("always"),
            PartialToken::raw("0"),
            PartialToken::raw("0"),
        ]));
        Ok(())
    }
}

#[derive(Debug)]
struct TwoLines;

impl Instruction<TestProcessor> for TwoLines {
    fn lower(
        &self,
        _ctx: &mut LowerContext<TestProcessor>,
        out: &mut PartialProgram<TestProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(vec![PartialToken::raw("noop")]));
        out.push_line(PartialLine::new(vec![PartialToken::raw("end")]));
        Ok(())
    }
}

#[test]
fn labels_resolve_after_multiline_lowering() {
    let processor = Processor::<TestProcessor>::new();
    let target = processor.label();

    processor.push(RawLine(&["set", "x", "1"]));
    processor.push(TwoLines);
    processor.push(JumpTo(target.clone()));
    processor.place(target);
    processor.push(RawLine(&["set", "x", "2"]));

    let output = processor.emit().expect("emit succeeds");

    assert_eq!(output, "set x 1\nnoop\nend\njump 4 always 0 0\nset x 2");
}

#[test]
fn unplaced_label_is_an_emit_error() {
    let processor = Processor::<TestProcessor>::new();
    let missing = processor.label();
    processor.push(JumpTo(missing));

    let error = processor.emit().expect_err("label is not placed");

    assert!(error.to_string().contains("unplaced label"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo test -p mlcg_core --test core_emit labels_resolve_after_multiline_lowering unplaced_label_is_an_emit_error
```

Expected: FAIL because instruction, label, lowering, and emit APIs are not defined.

- [ ] **Step 3: Add error types**

Create `crates/mlcg_core/src/error.rs`:

```rust
use snafu::Snafu;

use crate::{LabelId, ValueId};

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum LowerError {
    #[snafu(display("instruction lowering failed"))]
    Instruction,
}

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum EmitError {
    #[snafu(display("unplaced label {:?}", label))]
    UnplacedLabel { label: LabelId },
    #[snafu(display("unknown value {:?}", value))]
    UnknownValue { value: ValueId },
    #[snafu(display("failed to lower instruction"))]
    Lower { source: LowerError },
}
```

- [ ] **Step 4: Add label type**

Create `crates/mlcg_core/src/label.rs`:

```rust
use std::marker::PhantomData;

use crate::LabelId;

#[derive(Debug)]
pub struct Label<P> {
    pub(crate) id: LabelId,
    pub(crate) _processor: PhantomData<P>,
}

impl<P> Clone for Label<P> {
    fn clone(&self) -> Self {
        Self { id: self.id, _processor: PhantomData }
    }
}

impl<P> Label<P> {
    pub fn id(&self) -> LabelId {
        self.id
    }
}
```

- [ ] **Step 5: Add instruction and partial lowering types**

Create `crates/mlcg_core/src/instruction.rs`:

```rust
use std::fmt::Debug;

use crate::{label::Label, lower::{LowerContext, PartialProgram}, LowerError};

pub trait Instruction<P>: Debug + Send + Sync + 'static {
    fn lower(
        &self,
        ctx: &mut LowerContext<P>,
        out: &mut PartialProgram<P>,
    ) -> Result<(), LowerError>;
}

#[derive(Debug)]
pub(crate) enum ProgramItem<P> {
    Instruction(Box<dyn Instruction<P>>),
    LabelPlacement(Label<P>),
}
```

Create `crates/mlcg_core/src/lower.rs`:

```rust
use std::{collections::HashMap, marker::PhantomData};

use crate::{Label, LabelId, Value, ValueId};

#[derive(Debug, Default)]
pub struct LowerContext<P> {
    pub(crate) _processor: PhantomData<P>,
}

#[derive(Debug, Default)]
pub struct PartialProgram<P> {
    lines: Vec<PartialLine<P>>,
}

impl<P> PartialProgram<P> {
    pub fn push_line(&mut self, line: PartialLine<P>) {
        self.lines.push(line);
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn lines(&self) -> &[PartialLine<P>] {
        &self.lines
    }
}

#[derive(Debug)]
pub struct PartialLine<P> {
    tokens: Vec<PartialToken<P>>,
}

impl<P> PartialLine<P> {
    pub fn new(tokens: Vec<PartialToken<P>>) -> Self {
        Self { tokens }
    }

    pub(crate) fn tokens(&self) -> &[PartialToken<P>] {
        &self.tokens
    }
}

#[derive(Debug)]
pub enum PartialToken<P> {
    Raw(String),
    Value(ValueId),
    Label(LabelId),
    #[doc(hidden)]
    Processor(PhantomData<P>),
}

impl<P> PartialToken<P> {
    pub fn raw(token: impl Into<String>) -> Self {
        Self::Raw(token.into())
    }

    pub fn value<T>(value: Value<P, T>) -> Self {
        Self::Value(value.id())
    }

    pub fn label(label: Label<P>) -> Self {
        Self::Label(label.id())
    }
}

#[derive(Debug, Default)]
pub(crate) struct LabelTable {
    lines: HashMap<LabelId, usize>,
}

impl LabelTable {
    pub(crate) fn insert(&mut self, label: LabelId, line: usize) {
        self.lines.insert(label, line);
    }

    pub(crate) fn get(&self, label: LabelId) -> Option<usize> {
        self.lines.get(&label).copied()
    }
}
```

- [ ] **Step 6: Add emitter**

Create `crates/mlcg_core/src/emit.rs`:

```rust
use std::collections::HashMap;

use snafu::{OptionExt, ResultExt};

use crate::{emit_error, lower::{LabelTable, PartialProgram, PartialToken}, EmitError, LabelId, ValueId};

#[derive(Debug, Default)]
pub(crate) struct NameAllocator {
    names: HashMap<ValueId, String>,
    used: HashMap<String, usize>,
}

impl NameAllocator {
    pub(crate) fn name_for(&mut self, id: ValueId, hint: Option<&str>) -> String {
        if let Some(name) = self.names.get(&id) {
            return name.clone();
        }

        let base = hint.filter(|s| !s.is_empty()).unwrap_or("__mlcg");
        let count = self.used.entry(base.to_string()).or_insert(0);
        let name = if *count == 0 && base != "__mlcg" {
            base.to_string()
        } else {
            format!("{base}_{count}")
        };
        *count += 1;
        self.names.insert(id, name.clone());
        name
    }
}

pub(crate) fn emit_partial<P>(
    partial: &PartialProgram<P>,
    labels: &LabelTable,
    value_names: &HashMap<ValueId, String>,
) -> Result<String, EmitError> {
    let mut out = String::new();
    for (line_index, line) in partial.lines().iter().enumerate() {
        if line_index > 0 {
            out.push('\n');
        }
        for (token_index, token) in line.tokens().iter().enumerate() {
            if token_index > 0 {
                out.push(' ');
            }
            match token {
                PartialToken::Raw(raw) => out.push_str(raw),
                PartialToken::Value(value) => {
                    let name = value_names
                        .get(value)
                        .context(emit_error::UnknownValueSnafu { value: *value })?;
                    out.push_str(name);
                }
                PartialToken::Label(label) => {
                    let line = labels
                        .get(*label)
                        .context(emit_error::UnplacedLabelSnafu { label: *label })?;
                    out.push_str(&line.to_string());
                }
                PartialToken::Processor(_) => {}
            }
        }
    }
    Ok(out)
}
```

- [ ] **Step 7: Wire processor program state, labels, push, and emit**

Replace `crates/mlcg_core/src/processor.rs` with:

```rust
use std::{collections::HashMap, marker::PhantomData, sync::{Arc, Mutex}};

use snafu::ResultExt;

use crate::{
    emit::{emit_partial, NameAllocator}, emit_error, instruction::ProgramItem,
    label::Label, lower::{LabelTable, LowerContext, PartialProgram}, value::{Any, Value},
    EmitError, Instruction, LabelId, ValueId,
};

#[derive(Debug)]
pub(crate) struct ProgramState<P> {
    pub(crate) next_value: u64,
    pub(crate) next_label: u64,
    pub(crate) values: HashMap<ValueId, Option<String>>,
    pub(crate) items: Vec<ProgramItem<P>>,
    pub(crate) _processor: PhantomData<P>,
}

#[derive(Debug)]
pub struct ProcessorHandle<P> {
    pub(crate) state: Arc<Mutex<ProgramState<P>>>,
}

impl<P> Clone for ProcessorHandle<P> {
    fn clone(&self) -> Self {
        Self { state: Arc::clone(&self.state) }
    }
}

#[derive(Debug, Clone)]
pub struct Processor<P> {
    handle: ProcessorHandle<P>,
}

impl<P> Default for Processor<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> Processor<P> {
    pub fn new() -> Self {
        Self {
            handle: ProcessorHandle {
                state: Arc::new(Mutex::new(ProgramState {
                    next_value: 0,
                    next_label: 0,
                    values: HashMap::new(),
                    items: Vec::new(),
                    _processor: PhantomData,
                })),
            },
        }
    }

    pub fn handle(&self) -> ProcessorHandle<P> {
        self.handle.clone()
    }

    pub fn new_value(&self) -> Value<P, Any> {
        self.allocate_value(None)
    }

    pub fn named(&self, name: impl Into<String>) -> Value<P, Any> {
        self.allocate_value(Some(name.into()))
    }

    pub fn label(&self) -> Label<P> {
        let mut state = self.handle.state.lock().expect("program state mutex poisoned");
        let id = LabelId(state.next_label);
        state.next_label += 1;
        Label { id, _processor: PhantomData }
    }

    pub fn place(&self, label: Label<P>) {
        self.handle.push_item(ProgramItem::LabelPlacement(label));
    }

    pub fn push<I>(&self, instruction: I)
    where
        I: Instruction<P>,
    {
        self.handle.push_item(ProgramItem::Instruction(Box::new(instruction)));
    }

    pub fn emit(&self) -> Result<String, EmitError> {
        self.handle.emit()
    }

    fn allocate_value(&self, name_hint: Option<String>) -> Value<P, Any> {
        let mut state = self.handle.state.lock().expect("program state mutex poisoned");
        let id = ValueId(state.next_value);
        state.next_value += 1;
        state.values.insert(id, name_hint.clone());
        drop(state);
        Value { id, handle: self.handle.clone(), name_hint, _type: PhantomData }
    }
}

impl<P> ProcessorHandle<P> {
    pub(crate) fn push_item(&self, item: ProgramItem<P>) {
        let mut state = self.state.lock().expect("program state mutex poisoned");
        state.items.push(item);
    }

    pub(crate) fn emit(&self) -> Result<String, EmitError> {
        let state = self.state.lock().expect("program state mutex poisoned");
        let mut partial = PartialProgram::default();
        let mut labels = LabelTable::default();
        let mut lower_ctx = LowerContext::default();

        for item in &state.items {
            match item {
                ProgramItem::Instruction(instruction) => instruction
                    .lower(&mut lower_ctx, &mut partial)
                    .context(emit_error::LowerSnafu)?,
                ProgramItem::LabelPlacement(label) => labels.insert(label.id(), partial.line_count()),
            }
        }

        let mut allocator = NameAllocator::default();
        let mut value_names = HashMap::new();
        for (id, hint) in &state.values {
            value_names.insert(*id, allocator.name_for(*id, hint.as_deref()));
        }

        emit_partial(&partial, &labels, &value_names)
    }
}
```

- [ ] **Step 8: Update core exports**

Replace `crates/mlcg_core/src/lib.rs` with:

```rust
#![forbid(unsafe_code)]

pub mod emit;
pub mod error;
pub mod id;
pub mod instruction;
pub mod label;
pub mod lower;
pub mod processor;
pub mod value;

pub use error::{EmitError, LowerError};
pub use id::{LabelId, ValueId};
pub use instruction::Instruction;
pub use label::Label;
pub use lower::{LowerContext, PartialLine, PartialProgram, PartialToken};
pub use processor::{Processor, ProcessorHandle};
pub use value::{Any, Value};
```

- [ ] **Step 9: Run tests and commit**

```bash
cargo fmt --all
cargo test -p mlcg_core --test core_emit
cargo test --workspace --all-targets
git add crates/mlcg_core
git commit -m "feat(core): add labels lowering and emission"
```

## Task 4: Add thread-safety behavior test

**Files:**
- Test: `crates/mlcg_core/tests/threading.rs`

- [ ] **Step 1: Write thread push test**

Create `crates/mlcg_core/tests/threading.rs`:

```rust
use std::thread;

use mlcg_core::{Instruction, LowerContext, PartialLine, PartialProgram, PartialToken, Processor};

struct ThreadProcessor;

#[derive(Debug)]
struct Noop;

impl Instruction<ThreadProcessor> for Noop {
    fn lower(
        &self,
        _ctx: &mut LowerContext<ThreadProcessor>,
        out: &mut PartialProgram<ThreadProcessor>,
    ) -> Result<(), mlcg_core::LowerError> {
        out.push_line(PartialLine::new(vec![PartialToken::raw("noop")]));
        Ok(())
    }
}

#[test]
fn processor_accepts_pushes_from_multiple_threads() {
    let processor = Processor::<ThreadProcessor>::new();

    let mut handles = Vec::new();
    for _ in 0..4 {
        let processor = processor.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..25 {
                processor.push(Noop);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread joins");
    }

    let output = processor.emit().expect("emit succeeds");
    assert_eq!(output.lines().count(), 100);
}
```

- [ ] **Step 2: Run test to verify behavior**

Run:

```bash
cargo test -p mlcg_core --test threading processor_accepts_pushes_from_multiple_threads
```

Expected: PASS. If it fails due to `Send`/`Sync` bounds, add `P: Send + Sync + 'static` bounds only where the compiler requires them for stored instructions and handles.

- [ ] **Step 3: Run workspace tests and commit**

```bash
cargo fmt --all
cargo test --workspace --all-targets
git add crates/mlcg_core/tests/threading.rs crates/mlcg_core/src
git commit -m "test(core): cover threaded instruction pushes"
```

## Task 5: Implement facade `processor!()` and compile-time processor isolation

**Files:**
- Modify: `crates/mlcg/src/lib.rs`
- Create: `crates/mlcg/tests/processor_macro.rs`
- Create: `crates/mlcg/tests/ui/mixed_processors.rs`
- Create: `crates/mlcg/tests/ui/pass_same_processor.rs`

- [ ] **Step 1: Write runtime and trybuild tests**

Create `crates/mlcg/tests/processor_macro.rs`:

```rust
#[test]
fn processor_macro_creates_processors() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_same_processor.rs");
    t.compile_fail("tests/ui/mixed_processors.rs");
}
```

Create `crates/mlcg/tests/ui/pass_same_processor.rs`:

```rust
use mlcg::prelude::*;

fn accepts_same<P>(_: Value<P>, _: Value<P>) {}

fn main() {
    let processor = processor!();
    let a = processor.new_value();
    let b = processor.named("b");
    accepts_same(a, b);
}
```

Create `crates/mlcg/tests/ui/mixed_processors.rs`:

```rust
use mlcg::prelude::*;

fn accepts_same<P>(_: Value<P>, _: Value<P>) {}

fn main() {
    let first = processor!();
    let second = processor!();
    accepts_same(first.new_value(), second.new_value());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p mlcg --test processor_macro
```

Expected: FAIL because `processor!` is not defined.

- [ ] **Step 3: Implement macro and prelude exports**

Replace `crates/mlcg/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub use mlcg_builtin as builtin;
pub use mlcg_core as core;

#[macro_export]
macro_rules! processor {
    () => {{
        struct MlcgProcessorMarker;
        $crate::core::Processor::<MlcgProcessorMarker>::new()
    }};
}

pub mod prelude {
    pub use crate::processor;
    pub use mlcg_builtin::latest::*;
    pub use mlcg_core::{Any, EmitError, Instruction, Label, LowerContext, PartialLine, PartialProgram, PartialToken, Processor, Value};
}
```

- [ ] **Step 4: Run trybuild once to create stderr**

Run:

```bash
cargo test -p mlcg --test processor_macro
```

Expected: FAIL and `trybuild` writes a `.stderr` file under `wip/`. Move the generated stderr into `crates/mlcg/tests/ui/mixed_processors.stderr`.

- [ ] **Step 5: Run tests and commit**

```bash
cargo fmt --all
cargo test -p mlcg --test processor_macro
cargo test --workspace --all-targets
git add crates/mlcg
git commit -m "feat(mlcg): add processor macro facade"
```

## Task 6: Implement macro generation from a representative manifest

**Files:**
- Create: `crates/mlcg_builtin_macros/src/manifest.rs`
- Create: `crates/mlcg_builtin_macros/src/generate.rs`
- Modify: `crates/mlcg_builtin_macros/src/lib.rs`
- Create: `crates/mlcg_builtin_macros/tests/macro_basic.rs`

- [ ] **Step 1: Write macro integration test**

Create `crates/mlcg_builtin_macros/tests/macro_basic.rs`:

```rust
use std::{fs, path::PathBuf};

#[test]
fn macro_generates_set_and_op_add() {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/basic_manifest.toml");
    fs::write(
        &manifest_path,
        r#"
version = "fixture"

[[instructions]]
family = "set"
variant = "set"
rust_name = "set"
emit = ["set", "$target", "$source"]
receiver = "target"
inputs = ["source"]
outputs = []

[[instructions]]
family = "op"
variant = "add"
rust_name = "op_add"
emit = ["op", "add", "$out", "$lhs", "$rhs"]
receiver = ""
inputs = ["lhs", "rhs"]
outputs = ["out"]
"#,
    )
    .expect("write manifest");

    let t = trybuild::TestCases::new();
    t.pass("tests/pass_macro_basic.rs");
}
```

Create `crates/mlcg_builtin_macros/tests/pass_macro_basic.rs`:

```rust
use mlcg_core::Processor;

mod generated {
    mlcg_builtin_macros::include_manifest!("tests/basic_manifest.toml");
}

use generated::prelude::*;

struct P;

fn main() {
    let processor = Processor::<P>::new();
    let x = processor.named("x");
    let y = processor.named("y");

    x.set(1);
    let out = processor.op_add(x.clone(), y.clone());
    processor.op_add_into(out, x, y);

    let text = processor.emit().unwrap();
    assert!(text.contains("set x 1"));
    assert!(text.contains("op add"));
}
```

- [ ] **Step 2: Run macro test to verify it fails**

Run:

```bash
cargo test -p mlcg_builtin_macros --test macro_basic
```

Expected: FAIL because the macro emits nothing.

- [ ] **Step 3: Add manifest model**

Create `crates/mlcg_builtin_macros/src/manifest.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Manifest {
    pub(crate) version: String,
    pub(crate) instructions: Vec<InstructionSpec>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InstructionSpec {
    pub(crate) family: String,
    pub(crate) variant: String,
    pub(crate) rust_name: String,
    pub(crate) emit: Vec<String>,
    #[serde(default)]
    pub(crate) receiver: String,
    #[serde(default)]
    pub(crate) inputs: Vec<String>,
    #[serde(default)]
    pub(crate) outputs: Vec<String>,
}
```

- [ ] **Step 4: Add generator implementation**

Create `crates/mlcg_builtin_macros/src/generate.rs` with code that:

```rust
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::manifest::{InstructionSpec, Manifest};

pub(crate) fn generate(manifest: &Manifest) -> TokenStream {
    let structs = manifest.instructions.iter().map(generate_instruction_struct);
    let processor_exts = manifest.instructions.iter().map(generate_processor_ext);
    let value_exts = manifest.instructions.iter().filter(|spec| !spec.receiver.is_empty()).map(generate_value_ext);
    let prelude_exports = manifest.instructions.iter().flat_map(|spec| {
        let processor_trait = processor_trait_name(spec);
        let value_trait = value_trait_name(spec);
        if spec.receiver.is_empty() {
            vec![quote! { #processor_trait }]
        } else {
            vec![quote! { #processor_trait }, quote! { #value_trait }]
        }
    });

    quote! {
        use mlcg_core::{Instruction, LowerContext, PartialLine, PartialProgram, PartialToken, Processor, Value};

        #[derive(Debug, Clone)]
        pub enum Arg<P> {
            Value(Value<P>),
            Raw(String),
        }

        impl<P> From<Value<P>> for Arg<P> {
            fn from(value: Value<P>) -> Self { Self::Value(value) }
        }

        impl<P> From<&Value<P>> for Arg<P> {
            fn from(value: &Value<P>) -> Self { Self::Value(value.clone()) }
        }

        impl<P> From<i32> for Arg<P> {
            fn from(value: i32) -> Self { Self::Raw(value.to_string()) }
        }

        impl<P> From<f64> for Arg<P> {
            fn from(value: f64) -> Self { Self::Raw(value.to_string()) }
        }

        impl<P> From<&str> for Arg<P> {
            fn from(value: &str) -> Self { Self::Raw(value.to_string()) }
        }

        fn push_arg<P>(tokens: &mut Vec<PartialToken<P>>, arg: &Arg<P>) {
            match arg {
                Arg::Value(value) => tokens.push(PartialToken::value(value.clone())),
                Arg::Raw(raw) => tokens.push(PartialToken::raw(raw.clone())),
            }
        }

        #(#structs)*
        #(#processor_exts)*
        #(#value_exts)*

        pub mod prelude {
            pub use super::{#(#prelude_exports,)*};
        }
    }
}

fn struct_name(spec: &InstructionSpec) -> Ident {
    to_pascal_ident(&spec.rust_name)
}

fn processor_trait_name(spec: &InstructionSpec) -> Ident {
    format_ident!("{}ProcessorExt", struct_name(spec))
}

fn value_trait_name(spec: &InstructionSpec) -> Ident {
    format_ident!("{}ValueExt", struct_name(spec))
}

fn to_pascal_ident(name: &str) -> Ident {
    let mut out = String::new();
    for part in name.split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    format_ident!("{}", out)
}
```

Then complete the three helper functions in the same file with explicit token generation:

- generated struct fields are `Arg<P>` for every unique `$name` referenced by `emit`;
- `lower` iterates `emit` tokens; literal tokens become `PartialToken::raw`; `$name` tokens call `push_arg` for the matching field;
- processor ext creates output values with `self.new_value()` when `outputs.len() == 1`; explicit `_into` methods take output args;
- value ext uses the receiver value as the receiver field and takes `inputs` as method parameters.

The implementation must panic during macro expansion with a descriptive message if an instruction has more than one auto output.

- [ ] **Step 5: Wire macro entrypoint**

Replace `crates/mlcg_builtin_macros/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

mod generate;
mod manifest;

#[proc_macro]
pub fn include_manifest(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as LitStr).value();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set");
    let manifest_path = std::path::Path::new(&manifest_dir).join(path);
    let text = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("failed to read manifest {}: {error}", manifest_path.display()));
    let manifest: manifest::Manifest = toml::from_str(&text)
        .unwrap_or_else(|error| panic!("failed to parse manifest {}: {error}", manifest_path.display()));
    generate::generate(&manifest).into()
}
```

- [ ] **Step 6: Run tests and commit**

```bash
cargo fmt --all
cargo test -p mlcg_builtin_macros --test macro_basic
cargo test --workspace --all-targets
git add crates/mlcg_builtin_macros
git commit -m "feat(macros): generate builtin APIs from manifest"
```

## Task 7: Add committed v158_1 representative manifest and builtin module

**Files:**
- Create: `crates/mlcg_builtin/manifests/v158_1.toml`
- Modify: `crates/mlcg_builtin/src/lib.rs`
- Create: `crates/mlcg_builtin/src/v158_1.rs`
- Create: `crates/mlcg_builtin/tests/v158_1.rs`

- [ ] **Step 1: Write builtin behavior test**

Create `crates/mlcg_builtin/tests/v158_1.rs`:

```rust
use mlcg_builtin::latest::prelude::*;
use mlcg_core::Processor;

struct P;

#[test]
fn generated_v158_1_api_emits_representative_mlog() {
    let processor = Processor::<P>::new();
    let x = processor.named("x");
    let y = processor.named("y");

    x.set(1);
    let sum = processor.op_add(x.clone(), y.clone());
    let inverted = processor.op_not(sum.clone());
    processor.op_add_into(inverted.clone(), sum, 2);

    let output = processor.emit().expect("emit succeeds");

    assert_eq!(
        output,
        "set x 1\nop add __mlcg_0 x y\nop not __mlcg_1 __mlcg_0 0\nop add __mlcg_1 __mlcg_0 2"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p mlcg_builtin --test v158_1
```

Expected: FAIL because the manifest and generated module are not wired.

- [ ] **Step 3: Create representative manifest**

Create `crates/mlcg_builtin/manifests/v158_1.toml`:

```toml
version = "158.1"

[[instructions]]
family = "set"
variant = "set"
rust_name = "set"
emit = ["set", "$target", "$source"]
receiver = "target"
inputs = ["source"]
outputs = []

[[instructions]]
family = "op"
variant = "add"
rust_name = "op_add"
emit = ["op", "add", "$out", "$lhs", "$rhs"]
receiver = ""
inputs = ["lhs", "rhs"]
outputs = ["out"]

[[instructions]]
family = "op"
variant = "not"
rust_name = "op_not"
emit = ["op", "not", "$out", "$input", "0"]
receiver = ""
inputs = ["input"]
outputs = ["out"]
```

- [ ] **Step 4: Wire generated module**

Create `crates/mlcg_builtin/src/v158_1.rs`:

```rust
mlcg_builtin_macros::include_manifest!("manifests/v158_1.toml");
```

Replace `crates/mlcg_builtin/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod v158_1;

pub mod latest {
    pub use super::v158_1::*;
}
```

- [ ] **Step 5: Run tests and commit**

```bash
cargo fmt --all
cargo test -p mlcg_builtin --test v158_1
cargo test --workspace --all-targets
git add crates/mlcg_builtin
git commit -m "feat(builtin): add v158_1 generated API slice"
```

## Task 8: Implement fixture-driven builtin generator

**Files:**
- Create: `crates/mlcg_builtin_gen/src/fixture_parser.rs`
- Modify: `crates/mlcg_builtin_gen/src/lib.rs`
- Modify: `crates/mlcg_builtin_gen/src/main.rs`
- Create: `crates/mlcg_builtin_gen/tests/fixtures/v158_1_logic.txt`
- Create: `crates/mlcg_builtin_gen/tests/fixture_parser.rs`

- [ ] **Step 1: Write generator fixture and test**

Create `crates/mlcg_builtin_gen/tests/fixtures/v158_1_logic.txt`:

```text
statement set receiver=target emit=set,$target,$source inputs=source outputs=
op add unary=false emit=op,add,$out,$lhs,$rhs inputs=lhs,rhs outputs=out
op not unary=true emit=op,not,$out,$input,0 inputs=input outputs=out
jump always emit=jump,$label,always,0,0 inputs= outputs= label=label
jump equal emit=jump,$label,equal,$lhs,$rhs inputs=lhs,rhs outputs= label=label
```

Create `crates/mlcg_builtin_gen/tests/fixture_parser.rs`:

```rust
use mlcg_builtin_gen::fixture_parser::parse_fixture_manifest;

#[test]
fn fixture_parser_outputs_manifest_toml() {
    let input = include_str!("fixtures/v158_1_logic.txt");
    let toml = parse_fixture_manifest("158.1", input).expect("fixture parses");

    assert!(toml.contains("version = \"158.1\""));
    assert!(toml.contains("rust_name = \"op_add\""));
    assert!(toml.contains("rust_name = \"op_not\""));
    assert!(toml.contains("emit = [\"op\", \"not\", \"$out\", \"$input\", \"0\"]"));
    assert!(toml.contains("rust_name = \"jump_always\""));
    assert!(toml.contains("rust_name = \"jump_equal\""));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test -p mlcg_builtin_gen --test fixture_parser
```

Expected: FAIL because `fixture_parser` does not exist.

- [ ] **Step 3: Implement fixture parser**

Create `crates/mlcg_builtin_gen/src/fixture_parser.rs`:

```rust
use serde::Serialize;
use snafu::{ensure, OptionExt, ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum FixtureParseError {
    #[snafu(display("line {line} is empty after trimming"))]
    EmptyLine { line: usize },
    #[snafu(display("line {line} is missing token {name}"))]
    MissingToken { line: usize, name: &'static str },
    #[snafu(display("line {line} has invalid key-value token {token}"))]
    InvalidKeyValue { line: usize, token: String },
    #[snafu(display("failed to serialize manifest"))]
    Serialize { source: toml::ser::Error },
}

#[derive(Debug, Serialize)]
struct Manifest {
    version: String,
    instructions: Vec<Instruction>,
}

#[derive(Debug, Serialize)]
struct Instruction {
    family: String,
    variant: String,
    rust_name: String,
    emit: Vec<String>,
    receiver: String,
    inputs: Vec<String>,
    outputs: Vec<String>,
}

pub fn parse_fixture_manifest(version: &str, input: &str) -> Result<String, FixtureParseError> {
    let mut instructions = Vec::new();
    for (index, raw_line) in input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let mut tokens = line.split_whitespace();
        let family = tokens.next().context(MissingTokenSnafu { line: line_no, name: "family" })?;
        let variant = tokens.next().context(MissingTokenSnafu { line: line_no, name: "variant" })?;
        let mut emit = Vec::new();
        let mut receiver = String::new();
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        for token in tokens {
            let (key, value) = token
                .split_once('=')
                .context(InvalidKeyValueSnafu { line: line_no, token: token.to_string() })?;
            match key {
                "emit" => emit = split_list(value),
                "receiver" => receiver = value.to_string(),
                "inputs" => inputs = split_list(value),
                "outputs" => outputs = split_list(value),
                "label" | "unary" => {}
                _ => return InvalidKeyValueSnafu { line: line_no, token: token.to_string() }.fail(),
            }
        }
        ensure!(!emit.is_empty(), MissingTokenSnafu { line: line_no, name: "emit" });
        instructions.push(Instruction {
            family: family.to_string(),
            variant: variant.to_string(),
            rust_name: format!("{}_{}", family, variant).replace("statement_", ""),
            emit,
            receiver,
            inputs,
            outputs,
        });
    }

    toml::to_string_pretty(&Manifest { version: version.to_string(), instructions })
        .context(SerializeSnafu)
}

fn split_list(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(ToString::to_string).collect()
    }
}
```

- [ ] **Step 4: Export parser and wire CLI**

Replace `crates/mlcg_builtin_gen/src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod fixture_parser;
```

Replace `crates/mlcg_builtin_gen/src/main.rs`:

```rust
use std::{env, fs, path::PathBuf};

use mlcg_builtin_gen::fixture_parser::parse_fixture_manifest;

fn main() {
    let mut args = env::args().skip(1);
    let version = args.next().unwrap_or_else(|| "158.1".to_string());
    let input = args.next().map(PathBuf::from).expect("usage: mlcg_builtin_gen <version> <fixture-input> <output-toml>");
    let output = args.next().map(PathBuf::from).expect("usage: mlcg_builtin_gen <version> <fixture-input> <output-toml>");

    let source = fs::read_to_string(&input).expect("read input fixture");
    let manifest = parse_fixture_manifest(&version, &source).expect("parse fixture");
    fs::write(&output, manifest).expect("write manifest");
}
```

- [ ] **Step 5: Run tests and commit**

```bash
cargo fmt --all
cargo test -p mlcg_builtin_gen --test fixture_parser
cargo test --workspace --all-targets
git add crates/mlcg_builtin_gen
git commit -m "feat(gen): parse fixture manifest source"
```

## Task 9: Add end-to-end example and final verification

**Files:**
- Create: `examples/basic.rs`
- Modify: `Cargo.toml` if Cargo does not discover the example automatically

- [ ] **Step 1: Write example**

Create `examples/basic.rs`:

```rust
use mlcg::prelude::*;

fn main() {
    let processor = processor!();

    let x = processor.named("x");
    let y = processor.named("y");

    x.set(1);
    let sum = processor.op_add(x.clone(), y);
    let inverted = processor.op_not(sum);
    inverted.set(0);

    println!("{}", processor.emit().expect("program emits"));
}
```

- [ ] **Step 2: Run example**

Run:

```bash
cargo run --example basic
```

Expected output contains these lines in order:

```text
set x 1
op add __mlcg_0 x y
op not __mlcg_1 __mlcg_0 0
set __mlcg_1 0
```

- [ ] **Step 3: Run final verification**

Run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

Expected: all commands exit 0 with no warnings or test failures.

- [ ] **Step 4: Commit final example**

```bash
git add examples/basic.rs
git commit -m "docs: add basic mlcg example"
```

## Self-review checklist

- Spec coverage: this plan covers the workspace, core `Processor`/`Value`/`Label`, thread-safe handle, multi-line lowering, delayed naming, builtin macro generation, version module, fixture generator, facade macro, and end-to-end example.
- Intentional slice boundary: this plan covers representative `set`, `op add`, `op not`, `jump always`, and `jump equal` structures in the manifest/generator design, but the first committed builtin behavior test uses `set` and `op` only. Add a `jump` behavior test during execution if macro support for label arguments is completed earlier than expected.
- Full v158.1 source extraction is not implemented by this vertical slice; fixture parsing establishes the generator architecture before real Mindustry Java parsing.
- The plan keeps `.gitignore` untouched.
- All Rust behavior tasks use test-first steps.
