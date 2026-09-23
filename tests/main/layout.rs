//! Representation tests for transparent and zero-sized policies.

use core::mem::{align_of, size_of};

use aequitas::systems::si::quantities::Time;
use horae::{
    integration::tableau::{Euler, Midpoint, Rk4},
    subcycling::SubcyclePlan,
    time::{Instant, StepSize},
};

#[test]
fn policy_markers_are_zero_sized() {
    assert_eq!(size_of::<Euler>(), 0);
    assert_eq!(size_of::<Midpoint>(), 0);
    assert_eq!(size_of::<Rk4>(), 0);
    assert_eq!(size_of::<SubcyclePlan<8>>(), 0);
}

#[test]
fn time_newtypes_preserve_aequitas_layout() {
    assert_eq!(size_of::<Instant<f32>>(), size_of::<Time<f32>>());
    assert_eq!(align_of::<Instant<f32>>(), align_of::<Time<f32>>());
    assert_eq!(size_of::<StepSize<f64>>(), size_of::<Time<f64>>());
    assert_eq!(align_of::<StepSize<f64>>(), align_of::<Time<f64>>());
}
