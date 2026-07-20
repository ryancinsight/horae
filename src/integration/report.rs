use eunomia::FloatElement;

use crate::time::{Instant, StepSize};

/// Allocation-free report for one completed explicit step.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct StepReport<T = f64> {
    start: Instant<T>,
    end: Instant<T>,
    step: StepSize<T>,
    evaluations: usize,
}

impl<T> StepReport<T>
where
    T: FloatElement,
{
    pub(crate) const fn new(
        start: Instant<T>,
        end: Instant<T>,
        step: StepSize<T>,
        evaluations: usize,
    ) -> Self {
        Self {
            start,
            end,
            step,
            evaluations,
        }
    }

    /// Start instant.
    #[must_use]
    pub const fn start(&self) -> Instant<T> {
        self.start
    }

    /// End instant.
    #[must_use]
    pub const fn end(&self) -> Instant<T> {
        self.end
    }

    /// Accepted fixed step.
    #[must_use]
    pub const fn step(&self) -> StepSize<T> {
        self.step
    }

    /// Number of right-hand-side evaluations.
    #[must_use]
    pub const fn evaluations(&self) -> usize {
        self.evaluations
    }
}
