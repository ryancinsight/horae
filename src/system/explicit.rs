use eunomia::FloatElement;

use crate::time::Instant;

/// Slice-based right-hand-side evaluation for an explicit first-order system.
///
/// Implementations write `dy/dt = f(t, y)` into `derivative`. Horae owns
/// neither the equation nor its state representation. Implementors should
/// reject incompatible slice lengths through their associated error.
pub trait ExplicitSystem<T>
where
    T: FloatElement,
{
    /// System-specific evaluation failure.
    type Error;

    /// Evaluate the derivative at `time` and `state`.
    ///
    /// # Errors
    ///
    /// Returns the implementation's associated error when evaluation cannot
    /// satisfy the system contract.
    fn evaluate(
        &self,
        time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error>;
}
