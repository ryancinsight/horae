//! The Robertson stiff benchmark through the implicit step.
//!
//! Robertson (1966) is the three-species chain whose timescales span roughly
//! seven orders of magnitude, so an explicit step is bounded at `h ~ 1e-7`.
//! This test integrates it with backward Euler at `h = 0.01` — five orders of
//! magnitude beyond that bound — and checks the invariants a stiff solver must
//! preserve: mass conservation and non-negativity.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use horae::{
    integration::{BackwardEuler, ImplicitWorkspace, step_implicit_into},
    system::{ExplicitSystem, ImplicitSystem},
    time::{Instant, StepSize},
};

/// Robertson's three reactions:
/// `y1 -> y2` (0.04), `y2 + y3 -> y1 + y3` (1e4), `2 y2 -> y3` (3e7).
struct Robertson;

impl ExplicitSystem<f64> for Robertson {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        let (y1, y2, y3) = (state[0], state[1], state[2]);
        let catalytic = 1e4_f64 * y2 * y3;
        derivative[0] = (-0.04_f64).mul_add(y1, catalytic);
        derivative[1] = (0.04_f64).mul_add(y1, -(catalytic + 3e7 * y2 * y2));
        derivative[2] = 3e7 * y2 * y2;
        Ok(())
    }
}

impl ImplicitSystem<f64> for Robertson {
    fn jacobian(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        jacobian: &mut [f64],
    ) -> Result<(), Self::Error> {
        let (y2, y3) = (state[1], state[2]);
        // Row-major; `d f_i / d y_j` at index `3 i + j`.
        jacobian.copy_from_slice(&[
            -0.04_f64,
            1e4 * y3,
            1e4 * y2,
            0.04,
            -1e4 * y3 - 6e7 * y2,
            -1e4 * y2,
            0.0,
            6e7 * y2,
            0.0,
        ]);
        Ok(())
    }
}

#[test]
fn backward_euler_holds_robertson_invariants_at_five_orders_above_the_explicit_bound() {
    let system = Robertson;
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite fixture");
    let step = StepSize::new(Time::from_base(0.01)).expect("invariant: positive fixture");
    let mut workspace = ImplicitWorkspace::<f64>::new(3).expect("nonzero dimension");

    let mut state = [1.0_f64, 0.0, 0.0];
    let mut output = [0.0_f64; 3];
    let mut time = start;
    for _ in 0..4000 {
        let report = step_implicit_into(
            &system,
            BackwardEuler,
            time,
            step,
            &state,
            &mut output,
            &mut workspace,
            1e-10,
        )
        .expect("Newton converges at every stiff step");
        state = output;
        time = report.end();
    }

    // Mass conservation: f sums to zero, so y1 + y2 + y3 = 1 is a linear
    // invariant backward Euler preserves at every step up to rounding.
    let total: f64 = state.iter().sum();
    assert!(
        (total - 1.0).abs() <= 1e-6,
        "mass conservation drifted: {total}"
    );

    // Non-negativity: the stiff fast mode must damp, never flip.
    for concentration in state {
        assert!(
            concentration >= 0.0,
            "concentration went negative: {concentration}"
        );
    }

    // y2 is the trace intermediate: it rises and relaxes to a small value by
    // steady state, the qualitative signature the benchmark is named for.
    assert!(
        state[1] < 1e-4,
        "intermediate y2 did not relax: {}",
        state[1]
    );
    assert!(state[0] > 0.7, "y1 departed its steady state: {}", state[0]);
}
