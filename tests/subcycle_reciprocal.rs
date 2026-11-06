//! Subcycle reciprocal precision and range regressions.

use aequitas::systems::si::quantities::Time;
use eunomia::{Bf16, F16, F32, F64};
use horae::{
    subcycling::{SubcycleError, SubcyclePlan},
    time::{StepSize, TimeError},
};

#[test]
fn half_precision_ratio_above_finite_count_retains_representable_reciprocal() {
    let parent = StepSize::new(Time::from_base(F16::from_bits(0x3c00)))
        .expect("one is a finite positive half-precision step");
    let child = SubcyclePlan::<65_520>::new()
        .expect("ratio is within the supported range")
        .child_step(parent)
        .expect("its reciprocal rounds to a positive half-precision value");
    assert_eq!(child.as_time().as_base().to_bits(), 0x0100);

    let doubled = StepSize::new(Time::from_base(F16::from_bits(0x4000)))
        .expect("two is a finite positive half-precision step");
    let scaled = SubcyclePlan::<65_520>::new()
        .expect("valid ratio")
        .child_step(doubled)
        .expect("scaled child remains representable");
    assert_eq!(scaled.as_time().as_base().to_bits(), 0x0200);
}

#[test]
fn single_precision_ratio_above_exact_integer_range_rounds_its_reciprocal() {
    let parent = StepSize::new(Time::from_base(1.0_f32)).expect("one is a finite positive step");
    let child = SubcyclePlan::<16_777_217>::new()
        .expect("ratio is within the supported range")
        .child_step(parent)
        .expect("the reciprocal is representable");
    assert_eq!(child.as_time().as_base().to_bits(), 0x337f_ffff);
}

#[test]
fn half_precision_rounds_the_exact_reciprocal_before_scaling() {
    let half = StepSize::new(Time::from_base(F16::from_bits(0x3c00))).expect("positive step");
    let child = SubcyclePlan::<2079>::new()
        .expect("valid ratio")
        .child_step(half)
        .expect("positive reciprocal");
    // At this exponent, F16 has spacing 2^-22. 2^22 / 2079 rounds to 2017.
    assert_eq!(child.as_time().as_base().to_bits(), 0x0fe1);
}

#[test]
fn bfloat_precision_rounds_the_exact_reciprocal_before_scaling() {
    let bfloat = StepSize::new(Time::from_base(Bf16::from_bits(0x3f80))).expect("positive step");
    let child = SubcyclePlan::<269>::new()
        .expect("valid ratio")
        .child_step(bfloat)
        .expect("positive reciprocal");
    // At this exponent, Bf16 has spacing 2^-16. 2^16 / 269 rounds to 244.
    assert_eq!(child.as_time().as_base().to_bits(), 0x3b74);
}

#[test]
fn single_precision_wrapper_uses_the_same_policy() {
    let single = StepSize::new(Time::from_base(F32(1.0))).expect("positive step");
    let single_child = SubcyclePlan::<16_777_217>::new()
        .expect("valid ratio")
        .child_step(single)
        .expect("positive reciprocal");
    assert_eq!(single_child.as_time().as_base().0.to_bits(), 0x337f_ffff);
}

#[test]
fn double_precision_wrapper_uses_the_same_policy() {
    let double = StepSize::new(Time::from_base(F64(1.0))).expect("positive step");
    let double_child = SubcyclePlan::<3>::new()
        .expect("valid ratio")
        .child_step(double)
        .expect("positive reciprocal");
    assert_eq!(
        double_child.as_time().as_base().0.to_bits(),
        0x3fd5_5555_5555_5555
    );
}

#[test]
fn supported_limits_and_reciprocal_underflow_keep_their_errors() {
    assert_eq!(SubcyclePlan::<0>::new(), Err(SubcycleError::ZeroRatio));
    #[cfg(target_pointer_width = "64")]
    assert_eq!(
        SubcyclePlan::<4_294_967_296>::new(),
        Err(SubcycleError::RatioTooLarge)
    );

    let half = StepSize::new(Time::from_base(F16::from_bits(0x3c00))).expect("positive step");
    let smallest = SubcyclePlan::<16_777_216>::new()
        .expect("valid ratio")
        .child_step(half)
        .expect("smallest half subnormal is positive");
    assert_eq!(smallest.as_time().as_base().to_bits(), 0x0001);

    let tie = SubcyclePlan::<33_554_432>::new()
        .expect("valid ratio")
        .child_step(half);
    assert!(matches!(
        tie,
        Err(SubcycleError::ChildStep(TimeError::NonPositiveStep))
    ));

    let single = StepSize::new(Time::from_base(1.0_f32)).expect("positive step");
    let max = SubcyclePlan::<4_294_967_295>::new()
        .expect("largest supported ratio")
        .child_step(single)
        .expect("positive reciprocal");
    assert_eq!(max.as_time().as_base().to_bits(), 0x2f80_0000);
}
