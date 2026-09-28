use aequitas::systems::si::quantities::Time;
use eunomia::FloatElement;

use crate::{
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

use super::{
    SliceRole, StepError, StepReport, StepWorkspace,
    error::ensure_dimension,
    tableau::{EmbeddedExplicitTableau, ExplicitTableau},
};

/// Caller-owned output slices for one embedded explicit step.
#[must_use]
pub struct EmbeddedOutputs<'output, T> {
    primary: &'output mut [T],
    error_estimate: &'output mut [T],
}

impl<'output, T> EmbeddedOutputs<'output, T> {
    /// Pair a primary output slice with its local-error estimate slice.
    pub const fn new(primary: &'output mut [T], error_estimate: &'output mut [T]) -> Self {
        Self {
            primary,
            error_estimate,
        }
    }
}

/// Advance `state` by one explicit Runge--Kutta step into `output`.
///
/// `method` is a zero-sized marker used only for type inference. All stage
/// storage comes from `workspace`; the function performs no allocation and
/// dispatches neither the system nor the tableau dynamically.
///
/// # Errors
///
/// Returns [`StepError::DimensionMismatch`] before evaluation when caller
/// slices and workspace disagree, [`StepError::Time`] when a stage or end
/// instant overflows to a non-finite value, or [`StepError::System`] when the
/// system rejects a stage evaluation.
pub fn step_into<T, System, Method, const STAGES: usize>(
    system: &System,
    _method: Method,
    start: Instant<T>,
    step: StepSize<T>,
    state: &[T],
    output: &mut [T],
    workspace: &mut StepWorkspace<T, STAGES>,
) -> Result<StepReport<T>, StepError<System::Error>>
where
    T: FloatElement,
    System: ExplicitSystem<T>,
    Method: ExplicitTableau<STAGES>,
{
    let dimension = state.len();
    ensure_dimension(SliceRole::Output, dimension, output.len())?;
    ensure_dimension(SliceRole::Workspace, dimension, workspace.dimension())?;

    workspace.prepare::<Method>();
    evaluate_stages::<T, System, STAGES>(system, start, step, state, workspace)?;
    let mut results = [output];
    combine::<T, 1, STAGES>(
        &mut results,
        state,
        *step.as_time().as_base(),
        &workspace.derivatives,
        &[&workspace.coefficients.b],
    );

    let end = start.advance(step).map_err(StepError::Time)?;
    Ok(StepReport::new(start, end, step, STAGES))
}

/// Advance `state` with an embedded explicit Runge--Kutta pair.
///
/// The primary higher-order result is written to `outputs.primary`. The
/// caller-owned `outputs.error_estimate` receives the primary result minus the
/// embedded result, so its componentwise absolute value is the local error
/// observation for an adaptive controller.
/// Both results reuse the same stage derivatives and perform no allocation.
///
/// # Errors
///
/// Returns [`StepError::DimensionMismatch`] before evaluation when the state,
/// output, error-estimate slice, or workspace dimensions disagree,
/// [`StepError::Time`] when a stage or end instant overflows to a non-finite
/// value, or [`StepError::System`] when the system rejects a stage evaluation.
pub fn step_embedded_into<T, System, Method, const STAGES: usize>(
    system: &System,
    _method: Method,
    start: Instant<T>,
    step: StepSize<T>,
    state: &[T],
    outputs: EmbeddedOutputs<'_, T>,
    workspace: &mut StepWorkspace<T, STAGES>,
) -> Result<StepReport<T>, StepError<System::Error>>
where
    T: FloatElement,
    System: ExplicitSystem<T>,
    Method: EmbeddedExplicitTableau<STAGES>,
{
    let EmbeddedOutputs {
        primary: output,
        error_estimate,
    } = outputs;
    let dimension = state.len();
    ensure_dimension(SliceRole::Output, dimension, output.len())?;
    ensure_dimension(SliceRole::ErrorEstimate, dimension, error_estimate.len())?;
    ensure_dimension(SliceRole::Workspace, dimension, workspace.dimension())?;

    workspace.prepare::<Method>();
    workspace.prepare_embedded::<Method>();
    evaluate_stages::<T, System, STAGES>(system, start, step, state, workspace)?;
    let mut results = [output, error_estimate];
    combine::<T, 2, STAGES>(
        &mut results,
        state,
        *step.as_time().as_base(),
        &workspace.derivatives,
        &[&workspace.coefficients.b, &workspace.embedded],
    );
    let [primary, embedded] = &mut results;
    for (result, estimate) in primary.iter().zip(embedded.iter_mut()) {
        *estimate = *result - *estimate;
    }

    let end = start.advance(step).map_err(StepError::Time)?;
    Ok(StepReport::new(start, end, step, STAGES))
}

fn evaluate_stages<T, System, const STAGES: usize>(
    system: &System,
    start: Instant<T>,
    step: StepSize<T>,
    state: &[T],
    workspace: &mut StepWorkspace<T, STAGES>,
) -> Result<(), StepError<System::Error>>
where
    T: FloatElement,
    System: ExplicitSystem<T>,
{
    let dimension = state.len();

    let step_value = *step.as_time().as_base();
    let start_value = *start.as_time().as_base();
    let coefficients = &workspace.coefficients;

    for stage in 0..STAGES {
        workspace.stage_state.copy_from_slice(state);

        for previous_stage in 0..stage {
            let factor = step_value * coefficients.a[stage][previous_stage];
            let derivative = stage_derivative(&workspace.derivatives, previous_stage, dimension);
            for (trial, slope) in workspace.stage_state.iter_mut().zip(derivative) {
                *trial = factor.scalar_fmadd(*slope, *trial);
            }
        }

        let stage_value = step_value.scalar_fmadd(coefficients.c[stage], start_value);
        let stage_time = Instant::new(Time::from_base(stage_value)).map_err(StepError::Time)?;
        let offset = stage * dimension;
        system
            .evaluate(
                stage_time,
                &workspace.stage_state,
                &mut workspace.derivatives[offset..offset + dimension],
            )
            .map_err(StepError::System)?;
    }

    Ok(())
}

/// Borrow the contiguous derivative vector for one stage.
///
/// The workspace lays its stage derivatives out as `STAGES` contiguous
/// `dimension`-length vectors, so one stage is a range of that buffer rather
/// than an allocation.
#[inline]
fn stage_derivative<T>(derivatives: &[T], stage: usize, dimension: usize) -> &[T] {
    let offset = stage * dimension;
    &derivatives[offset..offset + dimension]
}

/// Accumulate `N` weighted stage-derivative combinations over one workspace.
///
/// Every `outputs[n]` is reset to `state` and then receives
/// `state + step · Σ_stage weights[n][stage] · k_stage`, so `N == 1` is a plain
/// Runge--Kutta combination and `N == 2` produces the primary and embedded
/// pair from the same stage derivatives. `N` and `STAGES` are const-generic and
/// the weights are already converted to `T`, so the loops monomorphize with no
/// dynamic dispatch and no per-stage scalar conversion.
fn combine<T, const N: usize, const STAGES: usize>(
    outputs: &mut [&mut [T]; N],
    state: &[T],
    step_value: T,
    derivatives: &[T],
    weights: &[&[T; STAGES]; N],
) where
    T: FloatElement,
{
    let dimension = state.len();
    for output in outputs.iter_mut() {
        output.copy_from_slice(state);
    }
    for stage in 0..STAGES {
        let derivative = stage_derivative(derivatives, stage, dimension);
        for (output, stage_weights) in outputs.iter_mut().zip(weights) {
            let factor = step_value * stage_weights[stage];
            for (result, slope) in output.iter_mut().zip(derivative) {
                *result = factor.scalar_fmadd(*slope, *result);
            }
        }
    }
}
