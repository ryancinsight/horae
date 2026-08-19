use aequitas::systems::si::quantities::Time;
use eunomia::FloatElement;

use crate::{
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

use super::{
    SliceRole, StepError, StepReport, StepWorkspace,
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

    evaluate_stages::<T, System, Method, STAGES>(system, start, step, state, workspace)?;
    combine_output::<T, STAGES>(
        output,
        state,
        *step.as_time().as_base(),
        workspace,
        &Method::B,
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

    evaluate_stages::<T, System, Method, STAGES>(system, start, step, state, workspace)?;
    combine_embedded_output::<T, STAGES>(
        output,
        error_estimate,
        state,
        *step.as_time().as_base(),
        workspace,
        &Method::B,
        &Method::B_EMBEDDED,
    );

    let end = start.advance(step).map_err(StepError::Time)?;
    Ok(StepReport::new(start, end, step, STAGES))
}

fn evaluate_stages<T, System, Method, const STAGES: usize>(
    system: &System,
    start: Instant<T>,
    step: StepSize<T>,
    state: &[T],
    workspace: &mut StepWorkspace<T, STAGES>,
) -> Result<(), StepError<System::Error>>
where
    T: FloatElement,
    System: ExplicitSystem<T>,
    Method: ExplicitTableau<STAGES>,
{
    let dimension = state.len();

    let step_value = *step.as_time().as_base();
    let start_value = *start.as_time().as_base();

    for stage in 0..STAGES {
        workspace.stage_state.copy_from_slice(state);

        for previous_stage in 0..stage {
            let coefficient = T::from_f64(Method::A[stage][previous_stage]);
            let factor = step_value * coefficient;
            let offset = previous_stage * dimension;
            let derivative = &workspace.derivatives[offset..offset + dimension];
            for (trial, slope) in workspace.stage_state.iter_mut().zip(derivative) {
                *trial = factor.scalar_fmadd(*slope, *trial);
            }
        }

        let stage_fraction = T::from_f64(Method::C[stage]);
        let stage_value = step_value.scalar_fmadd(stage_fraction, start_value);
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

fn combine_output<T, const STAGES: usize>(
    output: &mut [T],
    state: &[T],
    step_value: T,
    workspace: &StepWorkspace<T, STAGES>,
    weights: &[f64; STAGES],
) where
    T: FloatElement,
{
    let dimension = state.len();
    output.copy_from_slice(state);
    for (stage, weight) in weights.iter().enumerate() {
        let weight = T::from_f64(*weight);
        let factor = step_value * weight;
        let offset = stage * dimension;
        let derivative = &workspace.derivatives[offset..offset + dimension];
        for (result, slope) in output.iter_mut().zip(derivative) {
            *result = factor.scalar_fmadd(*slope, *result);
        }
    }
}

fn combine_embedded_output<T, const STAGES: usize>(
    output: &mut [T],
    error_estimate: &mut [T],
    state: &[T],
    step_value: T,
    workspace: &StepWorkspace<T, STAGES>,
    primary_weights: &[f64; STAGES],
    embedded_weights: &[f64; STAGES],
) where
    T: FloatElement,
{
    let dimension = state.len();
    output.copy_from_slice(state);
    error_estimate.copy_from_slice(state);
    for (stage, (primary_weight, embedded_weight)) in
        primary_weights.iter().zip(embedded_weights).enumerate()
    {
        let primary_factor = step_value * T::from_f64(*primary_weight);
        let embedded_factor = step_value * T::from_f64(*embedded_weight);
        let offset = stage * dimension;
        let derivative = &workspace.derivatives[offset..offset + dimension];
        for ((primary, embedded), slope) in output
            .iter_mut()
            .zip(error_estimate.iter_mut())
            .zip(derivative)
        {
            *primary = primary_factor.scalar_fmadd(*slope, *primary);
            *embedded = embedded_factor.scalar_fmadd(*slope, *embedded);
        }
    }
    for (primary, embedded) in output.iter().zip(error_estimate.iter_mut()) {
        *embedded = *primary - *embedded;
    }
}

fn ensure_dimension<E>(
    role: SliceRole,
    expected: usize,
    actual: usize,
) -> Result<(), StepError<E>> {
    if expected == actual {
        Ok(())
    } else {
        Err(StepError::DimensionMismatch {
            role,
            expected,
            actual,
        })
    }
}
