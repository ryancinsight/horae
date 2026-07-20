use core::fmt;

use crate::time::TimeError;

/// Slice participating in a failed step-shape check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SliceRole {
    /// Caller-owned output state.
    Output,
    /// Workspace state dimension.
    Workspace,
}

/// Failure to allocate a structurally valid step workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorkspaceError {
    /// A zero-dimensional state has no integration contract.
    ZeroDimension,
    /// `dimension * STAGES` overflowed `usize`.
    CapacityOverflow,
    /// A zero-stage tableau cannot evaluate a system.
    ZeroStages,
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDimension => formatter.write_str("workspace dimension must be nonzero"),
            Self::CapacityOverflow => formatter.write_str("workspace stage capacity overflowed"),
            Self::ZeroStages => formatter.write_str("workspace stage count must be nonzero"),
        }
    }
}

impl core::error::Error for WorkspaceError {}

/// Failure to complete one explicit step.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum StepError<E> {
    /// A state, output, or workspace dimension differs from the input state.
    DimensionMismatch {
        /// Slice or workspace with the mismatched dimension.
        role: SliceRole,
        /// Required dimension.
        expected: usize,
        /// Observed dimension.
        actual: usize,
    },
    /// A stage time or end time was not representable as a finite instant.
    Time(TimeError),
    /// The explicit system rejected an evaluation.
    System(E),
}

impl<E> fmt::Display for StepError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionMismatch {
                role,
                expected,
                actual,
            } => write!(
                formatter,
                "{role:?} dimension mismatch: expected {expected}, observed {actual}"
            ),
            Self::Time(error) => write!(formatter, "step time is invalid: {error}"),
            Self::System(error) => write!(formatter, "system evaluation failed: {error}"),
        }
    }
}

impl<E> core::error::Error for StepError<E> where E: core::error::Error + 'static {}
