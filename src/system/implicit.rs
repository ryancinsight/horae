//! Backend-neutral implicit-system contract for stiff methods.

use eunomia::FloatElement;

use crate::time::Instant;

use super::ExplicitSystem;

/// A stiff-capable system: an [`ExplicitSystem`] that also supplies its
/// Jacobian `∂f/∂y` for implicit stepping.
///
/// An implicit tableau (backward Euler, BDF) assembles the Newton matrix
/// `I − h·∂f/∂y` from this Jacobian. A system without a representable
/// Jacobian simply does not implement this trait and remains explicit-only,
/// so the explicit path is untouched (ADR 0002).
pub trait ImplicitSystem<T>: ExplicitSystem<T>
where
    T: FloatElement,
{
    /// Write `∂f/∂y` at `time` and `state` into `jacobian`.
    ///
    /// `jacobian` is a caller-owned row-major buffer of `state.len()²`
    /// elements: entry `i * n + j` is `∂f_i/∂y_j` for `n = state.len()`.
    ///
    /// # Errors
    ///
    /// Returns the system's associated error when the Jacobian cannot be
    /// formed — for example, when the supplied buffer does not admit the
    /// system's square dimension.
    fn jacobian(
        &self,
        time: Instant<T>,
        state: &[T],
        jacobian: &mut [T],
    ) -> Result<(), Self::Error>;
}
