use eunomia::FloatElement;

use crate::time::{Instant, StepSize, TimeError};

use super::{EventClip, ScheduleError};

/// Validated nondecreasing borrowed event instants.
///
/// The schedule owns no event storage. Duplicate instants are admitted and
/// skipped together once the simulation reaches them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventSchedule<'a, T = f64> {
    events: &'a [Instant<T>],
}

impl<'a, T> EventSchedule<'a, T>
where
    T: FloatElement,
{
    /// Validate a borrowed event sequence.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError::Unsorted`] at the first decreasing pair.
    pub fn new(events: &'a [Instant<T>]) -> Result<Self, ScheduleError> {
        for (offset, pair) in events.windows(2).enumerate() {
            if pair[1] < pair[0] {
                return Err(ScheduleError::Unsorted { index: offset + 1 });
            }
        }
        Ok(Self { events })
    }

    /// Return the borrowed event storage unchanged.
    #[inline]
    #[must_use]
    pub const fn events(&self) -> &'a [Instant<T>] {
        self.events
    }

    /// Return the first event strictly later than `start`.
    ///
    /// Equal events are skipped, so duplicate notifications at the current
    /// instant cannot cause a zero-length step or repeatedly re-trigger the
    /// same boundary.
    #[must_use]
    pub fn next_after(&self, start: Instant<T>) -> Option<Instant<T>> {
        let mut low = 0;
        let mut high = self.events.len();
        while low < high {
            let middle = low + (high - low) / 2;
            if self.events[middle] <= start {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        self.events.get(low).copied()
    }

    /// Clip `proposed` at the next crossed event boundary.
    ///
    /// An event already equal to the proposed endpoint is reported but does
    /// not count as shortening. If the next event lies after the proposed
    /// endpoint, the original step is returned unchanged and no event is
    /// reported. When an event is reported, [`EventClip::event`] is the
    /// authoritative instant; [`EventClip::step`] contains its rounded
    /// positive duration from `start`.
    ///
    /// For finite same-sign `start` and event instants whose magnitudes differ
    /// by at most a factor of two, Sterbenz's theorem makes the subtraction
    /// exact, so adding the returned duration to `start` recovers the event.
    /// Outside that condition, the duration remains the correctly rounded
    /// result of the subtraction, but endpoint reconstruction is not an
    /// exactness guarantee.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::NonFinite`] when computing the proposed endpoint
    /// overflows. Valid schedule ordering makes the clipped difference
    /// positive.
    pub fn clip_step(
        &self,
        start: Instant<T>,
        proposed: StepSize<T>,
    ) -> Result<EventClip<T>, TimeError> {
        let end = start.advance(proposed)?;
        let Some(event) = self.next_after(start) else {
            return Ok(EventClip::new(proposed, None, false));
        };
        if event > end {
            return Ok(EventClip::new(proposed, None, false));
        }
        let shortened = event < end;
        let step = event.duration_since(start)?;
        Ok(EventClip::new(step, Some(event), shortened))
    }
}
