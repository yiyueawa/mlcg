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

    #[snafu(display("failed to lower instruction"))]
    Lower { source: LowerError },
}
