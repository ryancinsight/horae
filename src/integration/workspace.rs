use alloc::{boxed::Box, vec};

use eunomia::{FloatElement, NumericElement};

use super::WorkspaceError;

/// Reusable storage for a const-generic explicit tableau.
///
/// Construction allocates two contiguous buffers. Repeated
/// [`step_into`](super::step_into) calls neither allocate nor resize them.
pub struct StepWorkspace<T, const STAGES: usize> {
    pub(crate) derivatives: Box<[T]>,
    pub(crate) stage_state: Box<[T]>,
    dimension: usize,
}

impl<T, const STAGES: usize> StepWorkspace<T, STAGES>
where
    T: FloatElement,
{
    /// Allocate storage for `dimension` state variables.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError::ZeroDimension`] or
    /// [`WorkspaceError::ZeroStages`] for empty structural dimensions, and
    /// [`WorkspaceError::CapacityOverflow`] when the flattened stage capacity
    /// cannot be represented by `usize`.
    pub fn new(dimension: usize) -> Result<Self, WorkspaceError> {
        if dimension == 0 {
            return Err(WorkspaceError::ZeroDimension);
        }
        if STAGES == 0 {
            return Err(WorkspaceError::ZeroStages);
        }
        let stage_capacity = dimension
            .checked_mul(STAGES)
            .ok_or(WorkspaceError::CapacityOverflow)?;
        Ok(Self {
            derivatives: vec![<T as NumericElement>::ZERO; stage_capacity].into_boxed_slice(),
            stage_state: vec![<T as NumericElement>::ZERO; dimension].into_boxed_slice(),
            dimension,
        })
    }

    /// State dimension accepted by this workspace.
    #[inline]
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }

    /// Compile-time number of stage derivative vectors.
    #[inline]
    #[must_use]
    pub const fn stages(&self) -> usize {
        STAGES
    }
}
