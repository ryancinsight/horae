//! Value-semantic tests for embedded explicit Runge--Kutta stepping.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use horae::{
    integration::{
        SliceRole, StepError, StepWorkspace, step_embedded_into,
        tableau::{DormandPrince, EmbeddedExplicitTableau, ExplicitTableau},
    },
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

struct FourthDegreeRate;

impl ExplicitSystem<f64> for FourthDegreeRate {
    type Error = Infallible;

    fn evaluate(
        &self,
        time: Instant<f64>,
        _state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        derivative[0] = (*time.as_time().as_base()).powi(4);
        Ok(())
    }
}

fn run(rate: &FourthDegreeRate, step_value: f64) -> ([f64; 1], [f64; 1]) {
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite fixture");
    let step =
        StepSize::new(Time::from_base(step_value)).expect("invariant: positive finite fixture");
    let mut output = [0.0];
    let mut error = [0.0];
    let mut workspace = StepWorkspace::<f64, 7>::new(1).expect("invariant: nonzero workspace");
    let report = step_embedded_into(
        rate,
        DormandPrince,
        start,
        step,
        &[0.0],
        &mut output,
        &mut error,
        &mut workspace,
    )
    .expect("invariant: polynomial rate is infallible");

    assert_eq!(report.evaluations(), 7);
    assert_eq!(
        report.end(),
        start.advance(step).expect("invariant: finite end")
    );
    (output, error)
}

#[test]
fn dormand_prince_has_fifth_order_primary_and_fourth_order_embedded_result() {
    assert_eq!(DormandPrince::ORDER, 5);
    assert_eq!(DormandPrince::EMBEDDED_ORDER, 4);
    let primary_sum: f64 = DormandPrince::B.into_iter().sum();
    let embedded_sum: f64 = DormandPrince::B_EMBEDDED.into_iter().sum();
    assert!((primary_sum - 1.0).abs() <= 8.0 * f64::EPSILON);
    assert!((embedded_sum - 1.0).abs() <= 8.0 * f64::EPSILON);
}

#[test]
fn embedded_step_matches_polynomial_oracle_and_scales_error_estimate() {
    let rate = FourthDegreeRate;
    let (large_output, large_error) = run(&rate, 0.5);
    let (small_output, small_error) = run(&rate, 0.25);

    // The fifth-order primary result integrates t^4 exactly in exact
    // arithmetic. The bound allows the seven stage evaluations, coefficient
    // conversions, and fused multiply-add accumulation to round independently.
    let large_exact = 0.5_f64.powi(5) / 5.0;
    let small_exact = 0.25_f64.powi(5) / 5.0;
    let rounding_bound = 128.0 * f64::EPSILON;
    assert!((large_output[0] - large_exact).abs() <= rounding_bound);
    assert!((small_output[0] - small_exact).abs() <= rounding_bound);

    // The embedded fourth-order quadrature error for t^4 is proportional to
    // h^5, so halving h reduces the signed estimate by 2^5.
    assert!(large_error[0].abs() > 0.0);
    let ratio = large_error[0] / small_error[0];
    assert!((ratio - 32.0).abs() <= 2.0e-12);
}

#[test]
fn embedded_step_validates_error_estimate_dimension_before_evaluation() {
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite fixture");
    let step = StepSize::new(Time::from_base(0.1)).expect("invariant: positive fixture");
    let mut output = [0.0];
    let mut workspace = StepWorkspace::<f64, 7>::new(1).expect("invariant: nonzero workspace");
    let result = step_embedded_into(
        &FourthDegreeRate,
        DormandPrince,
        start,
        step,
        &[0.0],
        &mut output,
        &mut [],
        &mut workspace,
    );

    assert_eq!(
        result,
        Err(StepError::DimensionMismatch {
            role: SliceRole::ErrorEstimate,
            expected: 1,
            actual: 0,
        })
    );
}
