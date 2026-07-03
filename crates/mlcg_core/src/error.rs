use snafu::Snafu;

use crate::{LabelId, ValueId};

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)))]
pub enum LowerError {
    #[snafu(display("instruction lowering failed"))]
    Instruction,
}

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)))]
pub enum EmitError {
    #[snafu(display("unplaced label {label:?}"))]
    UnplacedLabel { label: LabelId },

    #[snafu(display("duplicate label placement {label:?}"))]
    DuplicateLabelPlacement { label: LabelId },

    #[snafu(display("foreign label {label:?}"))]
    ForeignLabel { label: LabelId },

    #[snafu(display("foreign value {value:?}"))]
    ForeignValue { value: ValueId },

    #[snafu(display("unknown value {value:?}"))]
    UnknownValue { value: ValueId },

    #[snafu(display("empty value name for {value:?}"))]
    EmptyValueName { value: ValueId },

    #[snafu(display("blank value name for {value:?}"))]
    BlankValueName { value: ValueId },

    #[snafu(display("value name `{name}` contains whitespace for {value:?}"))]
    WhitespaceValueName { value: ValueId, name: String },

    #[snafu(display("duplicate value name `{name}`"))]
    DuplicateValueName { name: String },

    #[snafu(display("empty raw token"))]
    EmptyRawToken,

    #[snafu(display("blank raw token"))]
    BlankRawToken,

    #[snafu(display("raw token `{token}` contains whitespace"))]
    WhitespaceRawToken { token: String },

    #[snafu(display("empty line"))]
    EmptyLine,

    #[snafu(display("unresolved processor token"))]
    UnresolvedProcessorToken,

    #[snafu(display("failed to lower instruction"))]
    Lower { source: LowerError },
}
