//! Instrumented proof that stepping reuses all allocated storage.
//!
//! The measurement window counts allocations made by the calling thread only.
//! A process-wide counter is invalid here: libtest runs the test body on a
//! spawned thread while its main thread keeps inserting the running test into
//! its bookkeeping collections, so a process-wide window occasionally absorbs
//! those allocations and fails for reasons unrelated to the stepping path.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use allocation_counter::AllocationInfo;
use horae::{
    integration::{StepWorkspace, step_into, tableau::Rk4},
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

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

    let change = allocation_counter::measure(|| {
        for _ in 0..16 {
            let report = step_into(&Decay, Rk4, time, step, &state, &mut output, &mut workspace)
                .expect("invariant: infallible system");
            state.copy_from_slice(&output);
            time = report.end();
        }
    });

    assert_eq!(change, AllocationInfo::default());
}
