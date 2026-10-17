//! Verify the public quantity and scalar boundary with computed values.

use aequitas::systems::si::quantities::Time;
use eunomia::{FloatElement, NumericElement};
use horae::{
    adaptive::{AdaptiveController, StepDecision},
    time::{Instant, StepSize},
};

fn verify<T: FloatElement>() {
    let zero = <T as NumericElement>::ZERO;
    let one = <T as NumericElement>::ONE;
    let two = one + one;
    let four = two + two;
    let half = one / two;
    let quarter = half / two;
    let step = StepSize::new(Time::from_base(half)).expect("finite positive typed step");
    let start = Instant::new(Time::from_base(zero)).expect("finite typed instant");
    let end = start.advance(step).expect("finite advance");
    assert_eq!(*end.as_time().as_base(), half);
    assert_eq!(end.duration_since(start).expect("positive duration"), step);
    assert_eq!(
        *step
            .scaled(two)
            .expect("finite scaled step")
            .as_time()
            .as_base(),
        one
    );
    let controller =
        AdaptiveController::new(one, zero, one, quarter, two).expect("exact valid parameters");
    let report = controller.assess::<1>(step, four, one).expect("order one");
    assert_eq!(
        (report.decision(), report.normalized_error(), report.scale()),
        (StepDecision::Reject, four, half)
    );
    assert_eq!(*report.suggested_step().as_time().as_base(), quarter);
}

pub(crate) fn main() {
    verify::<f32>();
    verify::<f64>();
    verify::<eunomia::F32>();
    verify::<eunomia::F64>();
}
