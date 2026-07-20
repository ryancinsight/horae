use eunomia::FloatElement;

use crate::time::StepSize;

use super::StepDecision;

/// Allocation-free adaptive-controller result.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct AdaptiveReport<T = f64> {
    decision: StepDecision,
    normalized_error: T,
    scale: T,
    suggested_step: StepSize<T>,
}

impl<T> AdaptiveReport<T>
where
    T: FloatElement,
{
    pub(crate) const fn new(
        decision: StepDecision,
        normalized_error: T,
        scale: T,
        suggested_step: StepSize<T>,
    ) -> Self {
        Self {
            decision,
            normalized_error,
            scale,
            suggested_step,
        }
    }

    /// Acceptance decision for the trial state.
    #[must_use]
    pub const fn decision(&self) -> StepDecision {
        self.decision
    }

    /// Absolute error divided by the mixed absolute-relative tolerance.
    #[must_use]
    pub const fn normalized_error(&self) -> T {
        self.normalized_error
    }

    /// Multiplicative scale applied to the current step.
    #[must_use]
    pub const fn scale(&self) -> T {
        self.scale
    }

    /// Validated step suggested for the next trial.
    #[must_use]
    pub const fn suggested_step(&self) -> StepSize<T> {
        self.suggested_step
    }
}
