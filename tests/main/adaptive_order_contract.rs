//! Const-order boundaries retain their error and numerical contracts.

use aequitas::systems::si::quantities::Time;
use eunomia::{FloatElement, NumericElement};
use horae::{
    adaptive::{AdaptiveController, AdaptiveError, StepDecision},
    time::StepSize,
};

fn fixture<T: FloatElement>() -> (AdaptiveController<T>, StepSize<T>) {
    let zero = <T as NumericElement>::ZERO;
    let one = <T as NumericElement>::ONE;
    let two = one + one;
    let quarter = one / (two + two);
    (
        AdaptiveController::new(one, zero, one, quarter, two)
            .expect("valid exact controller parameters"),
        StepSize::new(Time::from_base(one / two)).expect("positive exact step"),
    )
}

fn verify_unsupported_orders<T: FloatElement>() {
    let (controller, step) = fixture::<T>();
    let one = <T as NumericElement>::ONE;
    assert_eq!(
        controller.assess::<0>(step, one, one),
        Err(AdaptiveError::InvalidOrder)
    );
    assert_eq!(
        controller.assess::<{ u32::MAX as usize }>(step, one, one),
        Err(AdaptiveError::InvalidOrder)
    );
    assert_eq!(
        controller.assess::<{ usize::MAX }>(step, one, one),
        Err(AdaptiveError::InvalidOrder)
    );
    // This neighboring unsupported value is representable only on 64-bit hosts.
    #[cfg(target_pointer_width = "64")]
    assert_eq!(
        controller.assess::<{ u32::MAX as usize + 1 }>(step, one, one),
        Err(AdaptiveError::InvalidOrder)
    );
}

fn verify_supported_orders<T: FloatElement>(epsilon: T) {
    let (controller, step) = fixture::<T>();
    let zero = <T as NumericElement>::ZERO;
    let one = <T as NumericElement>::ONE;
    let two = one + one;
    let four = two + two;
    let half = one / two;
    let quarter = half / two;
    // ORDER 1 gives factor 1/sqrt(4) = 1/2, hence suggested step 1/4.
    let control = controller.assess::<1>(step, four, one).expect("order one");
    assert_eq!(
        (
            control.decision(),
            control.normalized_error(),
            control.scale(),
            *control.suggested_step().as_time().as_base()
        ),
        (StepDecision::Reject, four, half, quarter)
    );
    for report in [
        controller.assess::<{ u32::MAX as usize - 2 }>(step, zero, one),
        controller.assess::<{ u32::MAX as usize - 1 }>(step, zero, one),
    ] {
        let report = report.expect("supported upper boundary with zero error");
        assert_eq!(
            (report.decision(), report.normalized_error(), report.scale()),
            (StepDecision::Accept, zero, two)
        );
        assert_eq!(*report.suggested_step().as_time().as_base(), one);
    }
    let report = controller
        .assess::<{ u32::MAX as usize - 1 }>(step, four, one)
        .expect("supported upper boundary with nonzero error");
    // For M = 2^32 - 1, 1 - 2^-31 < 4^(-1/M) < 1 - 2^-33.
    // Widen these analytic dyadic bounds by two native ulps at one.
    let mut dyadic = one;
    for _ in 0..31 {
        dyadic = dyadic / two;
    }
    let roundoff = two * epsilon;
    assert_eq!(report.decision(), StepDecision::Reject);
    assert_eq!(report.normalized_error(), four);
    assert!(report.scale() >= one - dyadic - roundoff);
    assert!(report.scale() <= one - dyadic / four + roundoff);
    assert!(report.scale() <= one);
    assert_eq!(
        *report.suggested_step().as_time().as_base(),
        half * report.scale()
    );
}

fn verify_observation_precedence<T: FloatElement>(nonfinite: T) {
    let (controller, step) = fixture::<T>();
    let one = <T as NumericElement>::ONE;
    for report in [
        controller.assess::<0>(step, nonfinite, one),
        controller.assess::<1>(step, one, nonfinite),
        controller.assess::<{ usize::MAX }>(step, nonfinite, one),
    ] {
        assert_eq!(report, Err(AdaptiveError::NonFiniteObservation));
    }
}

#[test]
fn unsupported_orders_return_the_documented_error() {
    verify_unsupported_orders::<f32>();
    verify_unsupported_orders::<f64>();
    verify_unsupported_orders::<eunomia::F32>();
    verify_unsupported_orders::<eunomia::F64>();
}

#[test]
fn supported_order_extremes_preserve_computed_values() {
    verify_supported_orders(f32::EPSILON);
    verify_supported_orders(f64::EPSILON);
    verify_supported_orders(eunomia::F32(f32::EPSILON));
    verify_supported_orders(eunomia::F64(f64::EPSILON));
}

#[test]
fn nonfinite_observations_keep_precedence_over_invalid_order() {
    verify_observation_precedence(f32::NAN);
    verify_observation_precedence(f64::INFINITY);
    verify_observation_precedence(eunomia::F32(f32::NEG_INFINITY));
    verify_observation_precedence(eunomia::F64(f64::NAN));
}
