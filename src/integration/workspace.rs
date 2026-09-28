use alloc::{boxed::Box, vec};

use eunomia::{FloatElement, NumericElement};

use super::{
    WorkspaceError,
    tableau::{EmbeddedExplicitTableau, ExplicitTableau},
};

/// Tableau coefficients converted once to the working scalar `T`.
///
/// The stepper's inner loops read already-converted scalars instead of
/// re-running `T::from_f64` on the same compile-time `f64` metadata for every
/// step. `a` and `c` drive stage evaluation; `b` is the primary output weight.
pub(crate) struct TableauCoefficients<T, const STAGES: usize> {
    pub(crate) a: [[T; STAGES]; STAGES],
    pub(crate) b: [T; STAGES],
    pub(crate) c: [T; STAGES],
}

impl<T, const STAGES: usize> TableauCoefficients<T, STAGES>
where
    T: FloatElement,
{
    /// Convert every coefficient of `Method` to `T`.
    pub(crate) fn convert<Method>() -> Self
    where
        Method: ExplicitTableau<STAGES>,
    {
        let mut a = [[<T as NumericElement>::ZERO; STAGES]; STAGES];
        for (row, coefficients) in a.iter_mut().enumerate() {
            for (entry, coefficient) in coefficients.iter_mut().zip(Method::A[row]) {
                *entry = T::from_f64(coefficient);
            }
        }
        Self {
            a,
            b: Method::B.map(T::from_f64),
            c: Method::C.map(T::from_f64),
        }
    }
}

/// Reusable storage for a const-generic explicit tableau.
///
/// Construction allocates two contiguous buffers. Repeated
/// [`step_into`](super::step_into) calls neither allocate nor resize them, and
/// the tableau coefficients are converted once into the workspace rather than
/// on every step.
pub struct StepWorkspace<T, const STAGES: usize> {
    pub(crate) derivatives: Box<[T]>,
    pub(crate) stage_state: Box<[T]>,
    pub(crate) coefficients: TableauCoefficients<T, STAGES>,
    pub(crate) embedded: [T; STAGES],
    coefficients_ready: bool,
    embedded_ready: bool,
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
            coefficients: TableauCoefficients {
                a: [[<T as NumericElement>::ZERO; STAGES]; STAGES],
                b: [<T as NumericElement>::ZERO; STAGES],
                c: [<T as NumericElement>::ZERO; STAGES],
            },
            embedded: [<T as NumericElement>::ZERO; STAGES],
            coefficients_ready: false,
            embedded_ready: false,
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

    /// Convert `Method`'s tableau coefficients into this workspace once.
    ///
    /// `ExplicitTableau` is sealed and Horae ships exactly one tableau per
    /// stage count, so the converted coefficients are valid for every call a
    /// workspace of this `STAGES` serves; conversion happens on first use and
    /// never again.
    pub(crate) fn prepare<Method>(&mut self)
    where
        Method: ExplicitTableau<STAGES>,
    {
        if !self.coefficients_ready {
            self.coefficients = TableauCoefficients::convert::<Method>();
            self.coefficients_ready = true;
        }
    }

    /// Convert `Method`'s embedded weights into this workspace once.
    ///
    /// See [`Self::prepare`] for why one conversion per workspace is sound.
    pub(crate) fn prepare_embedded<Method>(&mut self)
    where
        Method: EmbeddedExplicitTableau<STAGES>,
    {
        if !self.embedded_ready {
            self.embedded = Method::B_EMBEDDED.map(T::from_f64);
            self.embedded_ready = true;
        }
    }
}
