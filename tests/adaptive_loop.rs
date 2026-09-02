//! The estimator-to-controller composition (H-ADAPT-LOOP-002).
//!
//! Every existing `assess` call site passes a hand-written error value, so the
//! controller has only ever been verified against synthetic scalars: nothing
//! showed that the number `step_embedded_into` actually produces drives it to
//! the right decision, or that re-stepping at the suggested step then lands
//! inside tolerance. This closes that loop on a system with a closed-form
//! solution.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use horae::{
    adaptive::{AdaptiveController, StepDecision},
    integration::{EmbeddedOutputs, StepWorkspace, step_embedded_into, tableau::DormandPrince},
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

/// `y' = −y`, solution `y(t) = y0·e^(−t)`. Smooth, closed-form, and integrated
/// exactly by no tableau here, so the embedded pair reports a real local error.
#[derive(Clone, Copy)]
struct Decay;

impl ExplicitSystem<f64> for Decay {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        derivative[0] = -state[0];
        Ok(())
    }
}

/// One embedded step from `t = 0`, returning `(state, local error estimate)`.
fn embedded_step(step_value: f64, state: [f64; 1]) -> ([f64; 1], [f64; 1]) {
    let start = Instant::new(Time::from_base(0.0)).expect("invariant: finite start");
    let step = StepSize::new(Time::from_base(step_value)).expect("invariant: positive step");
    let mut output = [0.0];
    let mut error = [0.0];
    let mut workspace = StepWorkspace::<f64, 7>::new(1).expect("invariant: nonzero workspace");
    let _report = step_embedded_into(
        &Decay,
        DormandPrince,
        start,
        step,
        &state,
        EmbeddedOutputs::new(&mut output, &mut error),
        &mut workspace,
    )
    .expect("invariant: infallible system, matched dimensions");
    (output, error)
}

#[test]
fn a_real_error_estimate_drives_the_step_down_until_the_controller_accepts() {
    // Tolerances tight enough that a fifth-order estimate at this step exceeds
    // them by a wide margin, so the loop — not a single lucky proposal — is
    // what gets exercised.
    let controller = AdaptiveController::<f64>::new(1.0e-12, 1.0e-10, 0.9, 0.2, 5.0)
        .expect("invariant: valid controller");
    let initial = [1.0_f64];
    let mut step_value = 0.5_f64;

    let (coarse_state, coarse_error) = embedded_step(step_value, initial);
    let first = controller
        .assess::<5>(
            StepSize::new(Time::from_base(step_value)).expect("invariant: positive step"),
            coarse_error[0],
            coarse_state[0],
        )
        .expect("invariant: finite observation");
    assert_eq!(
        first.decision(),
        StepDecision::Reject,
        "a real fifth-order estimate at h = {step_value} must exceed a 1e-12          relative tolerance; got normalized error {}",
        first.normalized_error()
    );
    assert!(
        first.normalized_error() > 1.0,
        "a rejection means the normalized error exceeded one, got {}",
        first.normalized_error()
    );

    // Each rejection shrinks the step by at most the controller's clamp (0.2),
    // and the local error is O(h^5), so one rejection divides the normalized
    // error by at least 0.2^5 = 3125. From the first report's value, the
    // rejections still needed are bounded by log(e_0) / log(3125); eight is
    // that bound with room, and the loop asserting it is what proves the
    // controller drives itself to a decision rather than stalling.
    let bound = 8;
    let mut report = first;
    let mut rejections = 0;
    while report.decision() == StepDecision::Reject {
        rejections += 1;
        assert!(
            rejections <= bound,
            "the controller did not reach Accept within {bound} rejections;              last normalized error {}",
            report.normalized_error()
        );
        let retry = report.suggested_step();
        let retry_value = *retry.as_time().as_base();
        assert!(
            retry_value < step_value,
            "a rejected step must be proposed smaller: {retry_value} against {step_value}"
        );
        step_value = retry_value;
        let (state, error) = embedded_step(step_value, initial);
        report = controller
            .assess::<5>(retry, error[0], state[0])
            .expect("invariant: finite observation");
    }
    assert!(
        rejections >= 1,
        "the fixture must start outside tolerance for this to test the loop"
    );

    // The accepted state is the point of the exercise: it must be the true
    // solution to within the tolerance the controller just certified. The
    // local error the controller normalized bounds one step's deviation, and
    // the estimate is itself accurate to the next order, so twice it is a
    // sound bound on the distance to `e^(-h)`.
    let (accepted_state, accepted_error) = embedded_step(step_value, initial);
    let exact = (-step_value).exp();
    let deviation = (accepted_state[0] - exact).abs();
    assert!(
        deviation <= 2.0 * accepted_error[0].abs(),
        "accepted state {} is {deviation:.3e} from the closed-form {exact},          beyond twice the certified local error {:.3e}",
        accepted_state[0],
        accepted_error[0].abs()
    );
}

#[test]
fn a_step_the_controller_accepts_stays_accepted_when_it_is_re_taken() {
    // The composition must be deterministic: the same step on the same state
    // yields the same estimate and therefore the same decision. A controller
    // that reads uninitialized or accumulated workspace state would not.
    let controller = AdaptiveController::<f64>::new(1.0e-9, 1.0e-7, 0.9, 0.2, 5.0)
        .expect("invariant: valid controller");
    let initial = [1.0_f64];
    let step_value = 0.05_f64;
    let step = StepSize::new(Time::from_base(step_value)).expect("invariant: positive step");

    let (first_state, first_error) = embedded_step(step_value, initial);
    let (second_state, second_error) = embedded_step(step_value, initial);
    // Bit equality, deliberately: the claim is determinism, not closeness, so
    // the comparison is on representations rather than within a tolerance that
    // would hide a differing result.
    assert_eq!(
        first_state[0].to_bits(),
        second_state[0].to_bits(),
        "the embedded step must be a pure function of its inputs"
    );
    assert_eq!(
        first_error[0].to_bits(),
        second_error[0].to_bits(),
        "so must its error estimate"
    );

    let first = controller
        .assess::<5>(step, first_error[0], first_state[0])
        .expect("invariant: finite observation");
    let second = controller
        .assess::<5>(step, second_error[0], second_state[0])
        .expect("invariant: finite observation");
    assert_eq!(first.decision(), StepDecision::Accept);
    assert_eq!(second.decision(), first.decision());
    assert_eq!(
        second.normalized_error().to_bits(),
        first.normalized_error().to_bits()
    );
}
