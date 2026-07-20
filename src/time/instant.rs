use aequitas::systems::si::quantities::Time;
use eunomia::FloatElement;

use super::{StepSize, TimeError};

/// Finite simulation instant stored as an Aequitas time quantity.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Instant<T = f64>(Time<T>);

impl<T> Instant<T>
where
    T: FloatElement,
{
    /// Validate and construct a simulation instant.
    ///
    /// Negative instants are admitted because shifted time domains are valid;
    /// NaN and infinities are rejected so event ordering remains defined.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::NonFinite`] when `time` is NaN or infinite.
    pub fn new(time: Time<T>) -> Result<Self, TimeError> {
        if time.as_base().is_finite() {
            Ok(Self(time))
        } else {
            Err(TimeError::NonFinite)
        }
    }

    /// Borrow the underlying Aequitas time quantity.
    #[inline]
    #[must_use]
    pub const fn as_time(&self) -> &Time<T> {
        &self.0
    }

    /// Return the underlying Aequitas time quantity.
    #[inline]
    #[must_use]
    pub fn into_time(self) -> Time<T> {
        self.0
    }

    /// Advance this instant by a positive step.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::NonFinite`] when addition overflows the scalar's
    /// finite range.
    #[inline]
    pub fn advance(self, step: StepSize<T>) -> Result<Self, TimeError> {
        Self::new(self.0 + step.into_time())
    }

    /// Return the positive distance from `earlier` to this instant.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::NonPositiveStep`] when this instant is not later
    /// than `earlier`.
    pub fn duration_since(self, earlier: Self) -> Result<StepSize<T>, TimeError> {
        StepSize::new(self.0 - earlier.0)
    }
}
