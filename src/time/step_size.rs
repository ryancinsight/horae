use aequitas::systems::si::quantities::Time;
use eunomia::{FloatElement, NumericElement};

use super::TimeError;

/// Finite, strictly positive simulation step stored as Aequitas time.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct StepSize<T = f64>(Time<T>);

impl<T> StepSize<T>
where
    T: FloatElement,
{
    /// Validate and construct a simulation step.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::NonFinite`] for NaN or infinity and
    /// [`TimeError::NonPositiveStep`] for zero or a negative value.
    pub fn new(time: Time<T>) -> Result<Self, TimeError> {
        let value = *time.as_base();
        if !value.is_finite() {
            return Err(TimeError::NonFinite);
        }
        if value <= <T as NumericElement>::ZERO {
            return Err(TimeError::NonPositiveStep);
        }
        Ok(Self(time))
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

    /// Scale the step by a scalar and validate the result.
    ///
    /// # Errors
    ///
    /// Returns a [`TimeError`] when the scaled result is not finite and
    /// strictly positive.
    #[inline]
    pub fn scaled(self, factor: T) -> Result<Self, TimeError> {
        Self::new(self.0 * factor)
    }
}
