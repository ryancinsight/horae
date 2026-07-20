use aequitas::systems::si::quantities::Time;
use eunomia::FloatElement;

use crate::{
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

use super::{SliceRole, StepError, StepReport, StepWorkspace, tableau::ExplicitTableau};

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

    output.copy_from_slice(state);
    for stage in 0..STAGES {
        let weight = T::from_f64(Method::B[stage]);
        let factor = step_value * weight;
        let offset = stage * dimension;
        let derivative = &workspace.derivatives[offset..offset + dimension];
        for (result, slope) in output.iter_mut().zip(derivative) {
            *result = factor.scalar_fmadd(*slope, *result);
        }
    }

    let end = start.advance(step).map_err(StepError::Time)?;
    Ok(StepReport::new(start, end, step, STAGES))
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
