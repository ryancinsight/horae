use core::fmt;

use crate::time::TimeError;

use super::implicit::ImplicitStepError;

/// Slice participating in a failed step-shape check.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SliceRole {
    /// Caller-owned output state.
    Output,
    /// Caller-owned embedded local-error estimate.
    ErrorEstimate,
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
    /// A state, output, error, or workspace dimension differs from the input state.
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

/// A slice or workspace dimension that disagrees with the input state.
///
/// Both stepping paths share this check and payload. The `pub(crate)` type is
/// lifted into each path's public error enum through the [`From`] conversions
/// below, so the two error surfaces keep their own enum shapes while the
/// comparison and the payload layout live in one place.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DimensionMismatch {
    pub(crate) role: SliceRole,
    pub(crate) expected: usize,
    pub(crate) actual: usize,
}

/// Return `Ok(())` when `expected == actual`, else a [`DimensionMismatch`].
#[inline]
pub(crate) fn ensure_dimension(
    role: SliceRole,
    expected: usize,
    actual: usize,
) -> Result<(), DimensionMismatch> {
    if expected == actual {
        Ok(())
    } else {
        Err(DimensionMismatch {
            role,
            expected,
            actual,
        })
    }
}

impl<E> From<DimensionMismatch> for StepError<E> {
    fn from(mismatch: DimensionMismatch) -> Self {
        Self::DimensionMismatch {
            role: mismatch.role,
            expected: mismatch.expected,
            actual: mismatch.actual,
        }
    }
}

impl<E> From<DimensionMismatch> for ImplicitStepError<E> {
    fn from(mismatch: DimensionMismatch) -> Self {
        Self::DimensionMismatch {
            role: mismatch.role,
            expected: mismatch.expected,
            actual: mismatch.actual,
        }
    }
}
