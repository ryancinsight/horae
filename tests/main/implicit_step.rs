//! The implicit step: backward Euler integrates a linear decay to its closed
//! form, and the 2-D case exercises the dense Newton solve.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use horae::{
    integration::{BackwardEuler, ImplicitWorkspace, step_implicit_into},
    system::{ExplicitSystem, ImplicitSystem},
    time::{Instant, StepSize},
};

/// Diagonal linear decay `y' = diag(-rate)·y`.
struct Decay {
    rates: Vec<f64>,
}

impl ExplicitSystem<f64> for Decay {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        for (slope, (value, rate)) in derivative.iter_mut().zip(state.iter().zip(&self.rates)) {
            *slope = -rate * value;
        }
        Ok(())
    }
}

impl ImplicitSystem<f64> for Decay {
    fn jacobian(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        jacobian: &mut [f64],
    ) -> Result<(), Self::Error> {
        let n = state.len();
        for i in 0..n {
            for j in 0..n {
                jacobian[i * n + j] = if i == j { -self.rates[i] } else { 0.0 };
            }
        }
        Ok(())
    }
}

#[test]
fn backward_euler_recovers_linear_decay_closed_form() {
    // y' = -y: backward Euler gives y_{n+1} = y_n / (1 + h).
    let system = Decay { rates: vec![1.0] };
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite fixture");
    let step = StepSize::new(Time::from_base(0.5)).expect("invariant: positive fixture");
    let mut workspace = ImplicitWorkspace::<f64>::new(1).expect("nonzero dimension");

    let state = [1.0_f64];
    let mut output = [0.0_f64];
    let report = step_implicit_into(
        &system,
        BackwardEuler,
        start,
        step,
        &state,
        &mut output,
        &mut workspace,
        1e-8,
    )
    .expect("linear system converges in one Newton iteration");

    // y_1 = 1 / (1 + 0.5) = 2/3.
    assert_close(output[0], 2.0 / 3.0);
    // A constant Jacobian solves in exactly one Newton iteration: one
    // predictor evaluation plus one residual evaluation.
    // A constant Jacobian converges after the predictor and two residual checks:
    // the second check sees the exact solution and stops before solving.
    assert_eq!(report.evaluations(), 3);
}

#[test]
fn backward_euler_solves_a_diagonal_two_component_system() {
    // y' = diag(-1, -2)·y, decoupled: each component steps independently.
    let system = Decay {
        rates: vec![1.0, 2.0],
    };
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite fixture");
    let step = StepSize::new(Time::from_base(0.5)).expect("invariant: positive fixture");
    let mut workspace = ImplicitWorkspace::<f64>::new(2).expect("nonzero dimension");

    let state = [1.0_f64, 1.0];
    let mut output = [0.0_f64; 2];
    let report = step_implicit_into(
        &system,
        BackwardEuler,
        start,
        step,
        &state,
        &mut output,
        &mut workspace,
        1e-8,
    )
    .expect("decoupled linear system converges");

    // y_1 = 1/(1+0.5) and y_2 = 1/(1+1.0).
    assert_close(output[0], 2.0 / 3.0);
    assert_close(output[1], 0.5);
    // A constant Jacobian converges after the predictor and two residual checks:
    // the second check sees the exact solution and stops before solving.
    assert_eq!(report.evaluations(), 3);
}

fn assert_close(actual: f64, expected: f64) {
    let tolerance = 8.0 * f64::EPSILON * expected.abs().max(actual.abs()).max(1.0);
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual {actual} vs expected {expected}"
    );
}

#[test]
fn backward_euler_stays_bounded_on_a_stiff_two_scale_system() {
    // y' = diag(-1, -1000)·y: at h = 0.01 the fast component's explicit-Euler
    // amplification is 1 - 10 = -9, so an explicit step flips it negative and
    // diverges. Backward Euler is A-stable: the fast component maps to
    // 1/(1 + 10), still positive and bounded.
    let system = Decay {
        rates: vec![1.0, 1000.0],
    };
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite fixture");
    let step = StepSize::new(Time::from_base(0.01)).expect("invariant: positive fixture");
    let mut workspace = ImplicitWorkspace::<f64>::new(2).expect("nonzero dimension");

    let state = [1.0_f64, 1.0];
    let mut output = [0.0_f64; 2];
    let _report = step_implicit_into(
        &system,
        BackwardEuler,
        start,
        step,
        &state,
        &mut output,
        &mut workspace,
        1e-8,
    )
    .expect("A-stable step converges");

    // Slow component decays gently; the fast component is damped, not flipped.
    assert_close(output[0], 1.0 / 1.01);
    assert_close(output[1], 1.0 / 11.0);
    assert!(
        output[1] > 0.0,
        "fast component went negative: {}",
        output[1]
    );
}
