//! Value-semantic tests for embedded explicit Runge--Kutta stepping.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use eunomia::RealField;
use horae::{
    integration::{
        EmbeddedOutputs, SliceRole, StepError, StepWorkspace, step_embedded_into,
        tableau::{DormandPrince, EmbeddedExplicitTableau, ExplicitTableau},
    },
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

struct FourthDegreeRate;

impl<T: RealField> ExplicitSystem<T> for FourthDegreeRate {
    type Error = Infallible;

    fn evaluate(
        &self,
        time: Instant<T>,
        _state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        let t = *time.as_time().as_base();
        derivative[0] = t * t * t * t;
        Ok(())
    }
}

fn run<T: RealField>(rate: &FourthDegreeRate, step_value: T) -> ([T; 1], [T; 1]) {
    let start = Instant::new(Time::from_base(T::from_f64(0.0))).expect("invariant: finite fixture");
    let step =
        StepSize::new(Time::from_base(step_value)).expect("invariant: positive finite fixture");
    let mut output = [T::from_f64(0.0)];
    let mut error = [T::from_f64(0.0)];
    let mut workspace = StepWorkspace::<T, 7>::new(1).expect("invariant: nonzero workspace");
    let report = step_embedded_into(
        rate,
        DormandPrince,
        start,
        step,
        &[T::from_f64(0.0)],
        EmbeddedOutputs::new(&mut output, &mut error),
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

/// The polynomial oracle and the error-estimate scaling, at one scalar.
///
/// Both bounds are derived per scalar from that scalar's epsilon rather than
/// written once for `f64`: the seven stage evaluations, the coefficient
/// conversions, and the fused accumulation each round independently, so the
/// oracle bound is a fixed multiple of `T::EPSILON`. The scaling assertion is
/// looser at the narrower type for the same reason — the ratio of two
/// estimates carries the relative error of both.
fn assert_polynomial_oracle_and_error_scaling<T>(oracle_units: f64, ratio_units: f64)
where
    T: RealField,
{
    let rate = FourthDegreeRate;
    let large = T::from_f64(0.5);
    let small = T::from_f64(0.25);
    let (large_output, large_error) = run::<T>(&rate, large);
    let (small_output, small_error) = run::<T>(&rate, small);

    // The fifth-order primary result integrates t^4 exactly in exact
    // arithmetic, so the deviation is rounding alone.
    let large_exact = T::from_f64(0.5_f64.powi(5) / 5.0);
    let small_exact = T::from_f64(0.25_f64.powi(5) / 5.0);
    let bound = T::from_f64(oracle_units) * T::EPSILON;
    assert!(
        (large_output[0] - large_exact).abs() <= bound,
        "h = 0.5 integrated t^4 to {:?}, beyond {oracle_units} epsilon of the exact value",
        large_output[0]
    );
    assert!(
        (small_output[0] - small_exact).abs() <= bound,
        "h = 0.25 integrated t^4 to {:?}, beyond {oracle_units} epsilon of the exact value",
        small_output[0]
    );

    // The embedded fourth-order quadrature error for t^4 is proportional to
    // h^5, so halving h reduces the signed estimate by 2^5.
    assert!(
        large_error[0].abs() > T::from_f64(0.0),
        "a zero estimate would mean the embedded pair is not being evaluated"
    );
    let ratio = large_error[0] / small_error[0];
    let ratio_bound = T::from_f64(ratio_units) * T::EPSILON;
    assert!(
        (ratio - T::from_f64(32.0)).abs() <= ratio_bound,
        "halving h changed the estimate by {ratio:?}, not the 2^5 the h^5          scaling requires (within {ratio_units} epsilon)"
    );
}

#[test]
fn embedded_step_matches_polynomial_oracle_and_scales_error_estimate() {
    // The unit counts are the same derivation at both widths; only the epsilon
    // they multiply differs, which is the point of instantiating both.
    assert_polynomial_oracle_and_error_scaling::<f64>(128.0, 1.0e4);
    assert_polynomial_oracle_and_error_scaling::<f32>(128.0, 1.0e4);
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
        EmbeddedOutputs::new(&mut output, &mut []),
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
