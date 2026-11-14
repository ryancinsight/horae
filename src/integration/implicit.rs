//! Implicit one-step integration for stiff systems (ADR 0002).

use alloc::{boxed::Box, vec};
use core::fmt;

use eunomia::{NumericElement, RealField};

use crate::system::ImplicitSystem;
use crate::time::{Instant, StepSize, TimeError};

use super::{SliceRole, StepReport, WorkspaceError, error::ensure_dimension};

/// Nominal bound on Newton iterations before the step is declared
/// non-convergent. Backward Euler's Newton is contractive once the iterate is
/// near the solution, so the bound is a guard against a divergent predictor,
/// not a tuning knob.
const MAX_NEWTON_ITERATIONS: usize = 16;

/// Compile-time marker for an implicit one-step method.
pub trait ImplicitMethod {
    /// Formal accuracy order of the method.
    const ORDER: usize;
}

/// Backward Euler: `y_{n+1} = y_n + h·f(t_{n+1}, y_{n+1})`.
///
/// First order and A-stable, so its step is bounded by accuracy alone rather
/// than by the fastest system timescale — the property a stiff problem needs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackwardEuler;

impl ImplicitMethod for BackwardEuler {
    const ORDER: usize = 1;
}

/// The implicit Newton solve failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SolveError {
    /// Newton encountered a nonfinite residual or exhausted its iteration
    /// bound without satisfying the convergence tolerance.
    ///
    /// The bound guards against a divergent predictor rather than tuning the
    /// solve, so its value is an implementation detail and not part of this
    /// contract: a caller sees only that the step did not converge.
    NonConvergence,
    /// The Newton matrix was numerically singular.
    SingularMatrix,
}

impl fmt::Display for SolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonConvergence => formatter.write_str("Newton iteration did not converge"),
            Self::SingularMatrix => formatter.write_str("Newton matrix was singular"),
        }
    }
}

impl core::error::Error for SolveError {}

/// Failure to complete one implicit step.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ImplicitStepError<E> {
    /// A state, output, or workspace dimension differs from the input state.
    DimensionMismatch {
        /// Slice or workspace with the mismatched dimension.
        role: SliceRole,
        /// Required dimension.
        expected: usize,
        /// Observed dimension.
        actual: usize,
    },
    /// A stage or end instant was not representable as a finite time.
    Time(TimeError),
    /// The system rejected an evaluation or Jacobian formation.
    System(E),
    /// The implicit Newton solve failed.
    Solve(SolveError),
}

impl<E> fmt::Display for ImplicitStepError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionMismatch {
                role,
                expected,
                actual,
            } => write!(
                formatter,
                "{role:?} dimension mismatch: expected {expected}, observed {actual}"
            ),
            Self::Time(error) => write!(formatter, "step time is invalid: {error}"),
            Self::System(error) => write!(formatter, "system evaluation failed: {error}"),
            Self::Solve(error) => write!(formatter, "implicit solve failed: {error}"),
        }
    }
}

impl<E> core::error::Error for ImplicitStepError<E> where E: core::error::Error + 'static {}

/// Reusable scratch for the implicit Newton solve.
///
/// Construction allocates one residual vector and one dense square Jacobian
/// buffer. Repeated [`step_implicit_into`] calls neither allocate nor resize
/// them; the Jacobian buffer doubles as the in-place LU factorization.
pub struct ImplicitWorkspace<T> {
    residual: Box<[T]>,
    jacobian: Box<[T]>,
    dimension: usize,
}

impl<T> ImplicitWorkspace<T>
where
    T: RealField,
{
    /// Allocate storage for `dimension` state variables.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceError::ZeroDimension`] for an empty state and
    /// [`WorkspaceError::CapacityOverflow`] when the square Jacobian capacity
    /// cannot be represented by `usize`.
    pub fn new(dimension: usize) -> Result<Self, WorkspaceError> {
        if dimension == 0 {
            return Err(WorkspaceError::ZeroDimension);
        }
        let jacobian_capacity = dimension
            .checked_mul(dimension)
            .ok_or(WorkspaceError::CapacityOverflow)?;
        Ok(Self {
            residual: vec![<T as NumericElement>::ZERO; dimension].into_boxed_slice(),
            jacobian: vec![<T as NumericElement>::ZERO; jacobian_capacity].into_boxed_slice(),
            dimension,
        })
    }

    /// State dimension accepted by this workspace.
    #[inline]
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Advance `state` by one implicit step into `output`.
///
/// Uses the method named by `method` (backward Euler first) with a Newton
/// iteration: from the explicit-Euler predictor, it forms the residual
/// `g = y − y_n − h·f(t_{n+1}, y)`, assembles the Newton matrix `I − h·∂f/∂y`
/// from the system Jacobian, and solves the dense system for the correction
/// until every residual component is finite and the residual infinity norm
/// falls within `tolerance`.
///
/// # Acceptance rule
///
/// `tolerance` bounds the absolute residual in state units, and an iterate is
/// accepted only after at least one Newton correction. The predictor seeds
/// the iteration and is never itself the result. For `y' = λy` with
/// `z = hλ` the predictor is `(1 + z)·y_n` and its residual is
/// `g = −z²·y_n`: an absolute quantity that vanishes with `y_n` however far
/// the predictor lies from the backward-Euler value `y_n / (1 − z)`. For
/// `z < −2` the predictor has `|1 + z| > 1` and grows a decaying mode, yet
/// its residual passes whenever `z²·|y_n| <= tolerance`.
///
/// Newton's method is exact in one correction on a linear system with an
/// exact Jacobian (the residual is affine), so one mandatory correction
/// restores the backward-Euler value up to rounding. The rounding error
/// scales with the predictor magnitude, not with the result: for a decoupled
/// decay with `a = −z`, and without underflow (`y_n` and every intermediate
/// in the normal range of `T`), it is at most `4·ε·(|1 − a| + 1)·|y_n|`, with
/// `ε` the machine epsilon of `T`. That bound is below `|y_n|` only while
/// `a` is well under `1 / (4·ε)` (about `2·10⁶` in `f32`); beyond it an
/// accepted step can exceed `|y_n|` in magnitude or flip its sign even
/// though the residual is within `tolerance`, so the non-growth of a
/// decaying mode is not guaranteed for such stiffness. Coupled systems have
/// no per-component statement.
///
/// Subnormal contract: the routine relies on IEEE 754 gradual underflow and
/// states nothing under flush-to-zero. A result below the smallest positive
/// normal value `m` of `T` rounds to the subnormal spacing `δ = ε·m` (`2⁻¹⁴⁹`
/// in `f32`, `2⁻¹⁰⁷⁴` in `f64`), so it carries an absolute error of at most
/// `δ/2` in place of a relative one, while a sum or difference with a
/// subnormal result is exact. For a decoupled decay three roundings after the
/// predictor can fall there: the product `r·y`, its scaling by the step `h`,
/// and the division by `1 + a`. With `h` in base units the bound therefore
/// becomes, to first order in `ε`,
/// `4·ε·(|1 − a| + 1)·|y_n| + (2 + h)·δ/2`. For `y_n` of the order of `δ`
/// the absolute term dominates, so an accepted step can differ from
/// `y_n / (1 + a)` by a spacing or more however small `y_n` is.
///
/// On a nonlinear system the accepted iterate
/// has residual within `tolerance` after a correction, so choose `tolerance`
/// well below the magnitude of the state components it must resolve.
///
/// Every step evaluates the Jacobian and factors `I − h·∂f/∂y` at least
/// once, so a Jacobian error or an exactly singular matrix fails the step
/// even when the predictor already satisfies the residual test.
///
/// All matrix and vector scratch comes from `workspace`; the function performs
/// no allocation and dispatches neither the system nor the method dynamically.
///
/// # Errors
///
/// Returns [`ImplicitStepError::DimensionMismatch`] when the output or
/// workspace disagrees with the state, [`ImplicitStepError::Time`] when the end
/// instant is not finite, [`ImplicitStepError::System`] when the system rejects
/// an evaluation or Jacobian, and [`ImplicitStepError::Solve`] when Newton
/// encounters a nonfinite residual, does not converge, or the Newton matrix
/// is singular.
#[expect(
    clippy::too_many_arguments,
    reason = "the implicit step carries the explicit step's seven roles plus the Newton convergence tolerance"
)]
pub fn step_implicit_into<T, Method, System>(
    system: &System,
    _method: Method,
    start: Instant<T>,
    step: StepSize<T>,
    state: &[T],
    output: &mut [T],
    workspace: &mut ImplicitWorkspace<T>,
    tolerance: T,
) -> Result<StepReport<T>, ImplicitStepError<System::Error>>
where
    T: RealField,
    System: ImplicitSystem<T>,
    Method: ImplicitMethod,
{
    let dimension = state.len();
    ensure_dimension(SliceRole::Output, dimension, output.len())?;
    ensure_dimension(SliceRole::Workspace, dimension, workspace.dimension())?;

    let end = start.advance(step).map_err(ImplicitStepError::Time)?;
    let h = *step.as_time().as_base();

    // Predictor: the explicit-Euler point, y = y_n + h·f(t_n, y_n).
    system
        .evaluate(start, state, &mut workspace.residual)
        .map_err(ImplicitStepError::System)?;
    for (i, prediction) in output.iter_mut().enumerate() {
        *prediction = h.scalar_fmadd(workspace.residual[i], state[i]);
    }

    let mut evaluations = 1usize;
    let mut converged = false;
    let mut corrected = false;
    for _ in 0..MAX_NEWTON_ITERATIONS {
        evaluations += 1;

        // Residual g = y - y_n - h·f(t_{n+1}, y). The derivative is evaluated
        // into the residual buffer first, then folded into `g`.
        system
            .evaluate(end, output, &mut workspace.residual)
            .map_err(ImplicitStepError::System)?;
        for (i, residual) in workspace.residual.iter_mut().enumerate() {
            *residual = output[i] - state[i] - h * *residual;
        }

        // Convergence is the residual infinity norm: g -> 0 means the implicit
        // equation y = y_n + h·f is satisfied to the caller's tolerance.
        // NaN compares false, so an unchecked maximum can discard it and
        // incorrectly leave a zero norm. Every component must be finite.
        let mut residual_norm = <T as NumericElement>::ZERO;
        for residual in &workspace.residual {
            let magnitude = residual.abs();
            if !magnitude.is_finite() {
                return Err(ImplicitStepError::Solve(SolveError::NonConvergence));
            }
            if magnitude > residual_norm {
                residual_norm = magnitude;
            }
        }
        // The predictor is never accepted: see the acceptance rule above.
        if corrected && residual_norm <= tolerance {
            converged = true;
            break;
        }

        // Jacobian df/dy at (t_{n+1}, y), then the Newton matrix I - h·df/dy.
        system
            .jacobian(end, output, &mut workspace.jacobian)
            .map_err(ImplicitStepError::System)?;
        for (i, row) in workspace.jacobian.chunks_mut(dimension).enumerate() {
            for (j, entry) in row.iter_mut().enumerate() {
                let diagonal = if i == j {
                    <T as NumericElement>::ONE
                } else {
                    <T as NumericElement>::ZERO
                };
                *entry = diagonal - h * *entry;
            }
        }

        // Solve J·Δy = -g: negate the residual into the right-hand side, then
        // factorize the Jacobian in place and back-substitute.
        for residual in &mut workspace.residual {
            *residual = <T as NumericElement>::ZERO - *residual;
        }
        solve_dense(&mut workspace.jacobian, &mut workspace.residual)
            .map_err(ImplicitStepError::Solve)?;

        // Apply the correction, now held in the residual buffer as Δy.
        for (prediction, correction) in output.iter_mut().zip(&workspace.residual) {
            *prediction += *correction;
        }
        corrected = true;
    }

    if !converged {
        return Err(ImplicitStepError::Solve(SolveError::NonConvergence));
    }

    Ok(StepReport::new(start, end, step, evaluations))
}

/// Dense Gaussian elimination with partial pivoting, in place.
///
/// `matrix` is overwritten with its LU factorization and `rhs` with the
/// solution; the two are the caller's workspace buffers, so no allocation is
/// performed. The solve is exact up to rounding for a non-singular matrix and
/// returns [`SolveError::SingularMatrix`] when a pivot vanishes.
fn solve_dense<T>(matrix: &mut [T], rhs: &mut [T]) -> Result<(), SolveError>
where
    T: RealField,
{
    let dimension = rhs.len();
    for column in 0..dimension {
        // Partial pivot: the largest-magnitude entry at or below `column`.
        let mut pivot = column;
        let mut largest = matrix[column * dimension + column].abs();
        for row in (column + 1)..dimension {
            let magnitude = matrix[row * dimension + column].abs();
            if magnitude > largest {
                largest = magnitude;
                pivot = row;
            }
        }
        if largest <= <T as NumericElement>::ZERO {
            return Err(SolveError::SingularMatrix);
        }
        if pivot != column {
            for j in 0..dimension {
                matrix.swap(column * dimension + j, pivot * dimension + j);
            }
            rhs.swap(column, pivot);
        }

        // Eliminate below the pivot.
        for row in (column + 1)..dimension {
            let factor = matrix[row * dimension + column] / matrix[column * dimension + column];
            for j in (column + 1)..dimension {
                let pivot_entry = matrix[column * dimension + j];
                matrix[row * dimension + j] -= factor * pivot_entry;
            }
            rhs[row] -= factor * rhs[column];
        }
    }

    // Back substitution.
    for row in (0..dimension).rev() {
        let mut sum = rhs[row];
        for column in (row + 1)..dimension {
            sum -= matrix[row * dimension + column] * rhs[column];
        }
        rhs[row] = sum / matrix[row * dimension + row];
    }
    Ok(())
}
