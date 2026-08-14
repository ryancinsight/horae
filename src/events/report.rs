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
    ///
    /// When [`Self::event`] is `Some`, this value is the rounded positive
    /// duration from the clipping start to that event. The event instant is
    /// the authoritative boundary: reconstructing it with
    /// `start.advance(self.step())` is exact only when the subtraction that
    /// produced the duration satisfies the Sterbenz condition for the scalar
    /// (finite same-sign operands whose magnitudes differ by at most a factor
    /// of two).
    #[must_use]
    pub const fn step(&self) -> StepSize<T> {
        self.step
    }

    /// Authoritative event instant reached by this clipped step, when present.
    ///
    /// Consumers handling a boundary should use this value rather than
    /// reconstructing an endpoint from [`Self::step`]. The latter is an
    /// arithmetic duration and can differ by a representable ulp outside the
    /// Sterbenz exact-subtraction condition documented on [`Self::step`].
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
