use eunomia::FloatElement;

use crate::time::{Instant, StepSize};

/// Result of clipping a proposed step to the next scheduled event.
#[derive(Clone, Copy, Debug, PartialEq)]
#[must_use]
pub struct EventClip<T = f64> {
    step: StepSize<T>,
    event: Option<Instant<T>>,
    shortened: bool,
}

impl<T> EventClip<T>
where
    T: FloatElement,
{
    pub(crate) const fn new(step: StepSize<T>, event: Option<Instant<T>>, shortened: bool) -> Self {
        Self {
            step,
            event,
            shortened,
        }
    }

    /// Step ending no later than the next event.
    #[must_use]
    pub const fn step(&self) -> StepSize<T> {
        self.step
    }

    /// Event reached exactly at the clipped step endpoint, when present.
    #[must_use]
    pub const fn event(&self) -> Option<Instant<T>> {
        self.event
    }

    /// Whether the proposed step was shortened.
    #[must_use]
    pub const fn was_shortened(&self) -> bool {
        self.shortened
    }
}
