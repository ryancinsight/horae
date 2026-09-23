//! Instrumented proof that stepping reuses all allocated storage.

use core::convert::Infallible;
use std::alloc::System;

use aequitas::systems::si::quantities::Time;
use horae::{
    integration::{StepWorkspace, step_into, tableau::Rk4},
    system::ExplicitSystem,
    time::{Instant, StepSize},
};
use stats_alloc::{INSTRUMENTED_SYSTEM, Region, StatsAlloc};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

struct Decay;

impl ExplicitSystem<f64> for Decay {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        for (slope, value) in derivative.iter_mut().zip(state) {
            *slope = -*value;
        }
        Ok(())
    }
}

#[test]
fn repeated_steps_allocate_nothing_after_workspace_construction() {
    let mut workspace = StepWorkspace::<f64, 4>::new(4).expect("invariant: valid workspace");
    let mut state = [1.0, 2.0, 3.0, 4.0];
    let mut output = [0.0; 4];
    let step = StepSize::new(Time::from_base(0.01)).expect("invariant: positive fixture");
    let mut time = Instant::new(Time::from_base(0.0)).expect("invariant: finite fixture");

    let region = Region::new(GLOBAL);
    for _ in 0..16 {
        let report = step_into(&Decay, Rk4, time, step, &state, &mut output, &mut workspace)
            .expect("invariant: infallible system");
        state.copy_from_slice(&output);
        time = report.end();
    }
    let change = region.change();

    assert_eq!(change.allocations, 0);
    assert_eq!(change.reallocations, 0);
    assert_eq!(change.deallocations, 0);
}
