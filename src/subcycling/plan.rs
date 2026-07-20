use core::marker::PhantomData;

use eunomia::FloatElement;

use crate::time::StepSize;

use super::SubcycleError;

/// Zero-sized compile-time subcycle-ratio policy.
///
/// The ratio is the only modeled coupling policy. Horae does not invoke a
/// coarse or fine system and does not prescribe synchronization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubcyclePlan<const RATIO: usize>(PhantomData<()>);

impl<const RATIO: usize> SubcyclePlan<RATIO> {
    /// Validate and construct this const-generic ratio marker.
    ///
    /// # Errors
    ///
    /// Returns [`SubcycleError::ZeroRatio`] for zero and
    /// [`SubcycleError::RatioTooLarge`] above `u32::MAX`.
    pub fn new() -> Result<Self, SubcycleError> {
        validate_ratio::<RATIO>()?;
        Ok(Self(PhantomData))
    }

    /// Number of fine steps per coarse step.
    #[inline]
    #[must_use]
    pub const fn ratio(&self) -> usize {
        RATIO
    }

    /// Derive one fine step from a parent step.
    ///
    /// # Errors
    ///
    /// Returns a ratio validation error or [`SubcycleError::ChildStep`] when
    /// reduced-precision division underflows to zero.
    pub fn child_step<T>(&self, parent: StepSize<T>) -> Result<StepSize<T>, SubcycleError>
    where
        T: FloatElement,
    {
        let ratio = validate_ratio::<RATIO>()?;
        parent
            .scaled(T::from_f64(1.0 / f64::from(ratio)))
            .map_err(SubcycleError::ChildStep)
    }
}

fn validate_ratio<const RATIO: usize>() -> Result<u32, SubcycleError> {
    if RATIO == 0 {
        return Err(SubcycleError::ZeroRatio);
    }
    u32::try_from(RATIO).map_err(|_| SubcycleError::RatioTooLarge)
}
