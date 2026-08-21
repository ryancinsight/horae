//! Value-semantic fixed-step integration tests.

use core::{convert::Infallible, fmt};

use aequitas::systems::si::quantities::Time;
use eunomia::{NumericElement, RealField};
use horae::{
    integration::{
        SliceRole, StepError, StepWorkspace, step_into,
        tableau::{DormandPrince, Euler, ExplicitTableau, Midpoint, Rk4},
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

#[derive(Clone, Copy)]
struct ExponentialRate;

impl<T> ExplicitSystem<T> for ExponentialRate
where
    T: RealField,
{
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        derivative[0] = state[0];
        Ok(())
    }
}

fn march_exponential<T, Method, const STAGES: usize>(method: Method, subdivisions: usize) -> f64
where
    T: RealField,
    Method: ExplicitTableau<STAGES> + Copy,
{
    let subdivision_count = u32::try_from(subdivisions).expect("fixture count fits u32");
    let step_value = T::from_f64(1.0 / f64::from(subdivision_count));
    let step = StepSize::new(Time::from_base(step_value)).expect("invariant: positive step");
    let start = Instant::new(Time::from_base(T::from_f64(0.0))).expect("invariant: finite start");
    let mut time = start;
    let mut state = [T::from_f64(1.0)];
    let mut output = [<T as NumericElement>::ZERO];
    let mut workspace = StepWorkspace::<T, STAGES>::new(1).expect("invariant: nonzero workspace");

    for _ in 0..subdivisions {
        let report = step_into(
            &ExponentialRate,
            method,
            time,
            step,
            &state,
            &mut output,
            &mut workspace,
        )
        .expect("invariant: exponential system is infallible");
        state.copy_from_slice(&output);
        time = report.end();
    }

    state[0].to_f64()
}

fn observed_order<T, Method, const STAGES: usize>(
    method: Method,
    coarse_subdivisions: usize,
) -> (f64, f64, f64)
where
    T: RealField,
    Method: ExplicitTableau<STAGES> + Copy,
{
    let coarse = march_exponential::<T, Method, STAGES>(method, coarse_subdivisions);
    let fine = march_exponential::<T, Method, STAGES>(
        method,
        coarse_subdivisions
            .checked_mul(2)
            .expect("invariant: refinement count does not overflow"),
    );
    let exact = core::f64::consts::E;
    let coarse_error = (coarse - exact).abs();
    let fine_error = (fine - exact).abs();
    let rate = (coarse_error / fine_error).ln() / core::f64::consts::LN_2;
    (coarse_error, fine_error, rate)
}

#[expect(
    clippy::float_cmp,
    reason = "the rounded convergence class is the discrete oracle"
)]
fn assert_formal_order<T, Method, const STAGES: usize>(method: Method, coarse_subdivisions: usize)
where
    T: RealField,
    Method: ExplicitTableau<STAGES> + Copy,
{
    let (coarse_error, fine_error, observed) =
        observed_order::<T, Method, STAGES>(method, coarse_subdivisions);
    let formal =
        f64::from(u32::try_from(Method::ORDER).expect("invariant: tableau order fits u32"));

    // The dyadic refinement ratio is classified by its nearest formal order.
    // This is an order-class check, not a fabricated decimal accuracy claim:
    // the closed-form oracle supplies the value and the classification has no
    // tunable tolerance.
    assert!(coarse_error > fine_error, "refinement must reduce error");
    assert_eq!(observed.round(), formal, "observed order {observed}");
}

#[test]
fn refinement_recovers_each_tableau_order_against_closed_form_solution() {
    assert_formal_order::<f64, _, 1>(Euler, 16);
    assert_formal_order::<f64, _, 2>(Midpoint, 16);
    assert_formal_order::<f64, _, 4>(Rk4, 16);
    assert_formal_order::<f64, _, 7>(DormandPrince, 16);
}

#[test]
fn refinement_order_class_is_stable_for_f32() {
    // The f64 test covers all tableaus. At this finite dyadic range the
    // embedded fifth-order f32 signal is below its rounding floor, so the
    // f32 check is limited to the non-embedded tableaus with a resolvable
    // truncation signal; generic execution of Dormand--Prince remains covered
    // by the existing f32/f64 embedded tests.
    assert_formal_order::<f32, _, 1>(Euler, 4);
    assert_formal_order::<f32, _, 2>(Midpoint, 4);
    assert_formal_order::<f32, _, 4>(Rk4, 4);
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
