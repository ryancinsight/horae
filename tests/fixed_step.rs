//! Value-semantic fixed-step integration tests.

use core::{convert::Infallible, fmt};

use aequitas::systems::si::quantities::Time;
use eunomia::{NumericElement, RealField};
use horae::{
    integration::{
        SliceRole, StepError, StepWorkspace, step_into,
        tableau::{Euler, Midpoint, Rk4},
    },
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

#[derive(Clone, Copy)]
struct ConstantRate<T>(T);

impl<T> ExplicitSystem<T> for ConstantRate<T>
where
    T: RealField,
{
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        _state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        derivative.fill(self.0);
        Ok(())
    }
}

fn assert_constant_rate<T>()
where
    T: RealField,
{
    let start =
        Instant::new(Time::from_base(T::from_f64(0.25))).expect("invariant: finite start fixture");
    let step =
        StepSize::new(Time::from_base(T::from_f64(0.5))).expect("invariant: positive step fixture");
    let initial = [T::from_f64(1.0), T::from_f64(-2.0)];
    let expected = [T::from_f64(2.5), T::from_f64(-0.5)];
    let rate = ConstantRate(T::from_f64(3.0));

    let mut euler_output = [<T as NumericElement>::ZERO; 2];
    let mut euler_workspace = StepWorkspace::<T, 1>::new(2).expect("invariant: nonzero workspace");
    let euler_report = step_into(
        &rate,
        Euler,
        start,
        step,
        &initial,
        &mut euler_output,
        &mut euler_workspace,
    )
    .expect("invariant: infallible constant system");
    assert_eq!(euler_output, expected);
    assert_eq!(euler_report.evaluations(), 1);

    let mut midpoint_output = [<T as NumericElement>::ZERO; 2];
    let mut midpoint_workspace =
        StepWorkspace::<T, 2>::new(2).expect("invariant: nonzero workspace");
    let midpoint_report = step_into(
        &rate,
        Midpoint,
        start,
        step,
        &initial,
        &mut midpoint_output,
        &mut midpoint_workspace,
    )
    .expect("invariant: infallible constant system");
    assert_eq!(midpoint_output, expected);
    assert_eq!(midpoint_report.evaluations(), 2);

    let mut rk_output = [<T as NumericElement>::ZERO; 2];
    let mut rk_workspace = StepWorkspace::<T, 4>::new(2).expect("invariant: nonzero workspace");
    let rk_report = step_into(
        &rate,
        Rk4,
        start,
        step,
        &initial,
        &mut rk_output,
        &mut rk_workspace,
    )
    .expect("invariant: infallible constant system");
    for (actual, exact) in rk_output.into_iter().zip(expected) {
        // Four coefficient conversions, products, and additions contribute at
        // most 32 first-order rounding units at this fixture's result scale.
        let bound = T::from_f64(32.0) * T::from_f64(3.0) * T::from_f64(0.5) * T::EPSILON;
        assert!((actual - exact).abs() <= bound);
    }
    assert_eq!(rk_report.evaluations(), 4);
    assert_eq!(*rk_report.end().as_time().as_base(), T::from_f64(0.75));
}

#[test]
fn constant_rate_is_exact_for_each_method_and_scalar() {
    assert_constant_rate::<f32>();
    assert_constant_rate::<f64>();
}

struct TimeRate;

impl ExplicitSystem<f64> for TimeRate {
    type Error = Infallible;

    fn evaluate(
        &self,
        time: Instant<f64>,
        _state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        derivative.fill(*time.as_time().as_base());
        Ok(())
    }
}

#[test]
fn midpoint_uses_tableau_stage_time() {
    let start = Instant::new(Time::from_base(1.0)).expect("invariant: finite fixture");
    let step = StepSize::new(Time::from_base(0.5)).expect("invariant: positive fixture");
    let mut output = [0.0];
    let mut workspace = StepWorkspace::<f64, 2>::new(1).expect("invariant: valid workspace");
    let report = step_into(
        &TimeRate,
        Midpoint,
        start,
        step,
        &[0.0],
        &mut output,
        &mut workspace,
    )
    .expect("invariant: infallible system");
    assert_eq!(report.evaluations(), 2);
    // Integral of t over [1, 1.5] is exactly 0.625, and midpoint evaluates at
    // 1.25 so the method is exact for this linear time dependence.
    assert_eq!(output[0].to_bits(), 0.625_f64.to_bits());
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EvaluationFailure;

impl fmt::Display for EvaluationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("fixture rejected evaluation")
    }
}

impl core::error::Error for EvaluationFailure {}

struct FailingSystem;

impl ExplicitSystem<f64> for FailingSystem {
    type Error = EvaluationFailure;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        _state: &[f64],
        _derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        Err(EvaluationFailure)
    }
}

#[test]
fn shape_and_system_errors_preserve_failure_context() {
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite fixture");
    let step = StepSize::new(Time::from_base(0.1)).expect("invariant: positive fixture");
    let mut workspace = StepWorkspace::<f64, 1>::new(2).expect("invariant: valid workspace");
    let mut short_output = [0.0];

    let shape_error = step_into(
        &FailingSystem,
        Euler,
        start,
        step,
        &[1.0, 2.0],
        &mut short_output,
        &mut workspace,
    )
    .expect_err("output dimension differs");
    assert_eq!(
        shape_error,
        StepError::DimensionMismatch {
            role: SliceRole::Output,
            expected: 2,
            actual: 1,
        }
    );

    let mut output = [0.0, 0.0];
    let system_error = step_into(
        &FailingSystem,
        Euler,
        start,
        step,
        &[1.0, 2.0],
        &mut output,
        &mut workspace,
    )
    .expect_err("fixture always rejects evaluation");
    assert_eq!(system_error, StepError::System(EvaluationFailure));
}
