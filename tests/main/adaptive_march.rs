//! A reject-then-accept march with a recorded step history.
//!
//! `adaptive_loop.rs` drives one rejection loop from `t = 0` and stops at the
//! first accepted step. A consumer marches: it keeps `(time, state)` while a
//! trial is judged, discards a rejected trial, retries from the same instant
//! with the suggested step, and commits only an accepted one. This test is
//! that consumer, on `y' = -2ty` with the closed form `y(t) = y0·e^(-t²)`, and
//! it asserts what the caller can observe after every rejection.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use horae::{
    adaptive::{AdaptiveController, StepDecision},
    integration::{EmbeddedOutputs, StepWorkspace, step_embedded_into, tableau::DormandPrince},
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

const ABSOLUTE_TOLERANCE: f64 = 1.0e-10;
const RELATIVE_TOLERANCE: f64 = 1.0e-8;
const END_TIME: f64 = 3.0;
const FIRST_STEP: f64 = 0.5;
/// Formal order of the Dormand--Prince primary result, as declared by the tableau.
const ORDER: usize = 5;

#[derive(Clone, Copy)]
struct GaussianDecay;

/// `y' = -2ty`: the time argument enters, and the local error grows and then
/// shrinks along the march, so a step grown after an accept is later rejected.
impl ExplicitSystem<f64> for GaussianDecay {
    type Error = Infallible;

    fn evaluate(
        &self,
        time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        derivative[0] = -2.0 * time.as_time().as_base() * state[0];
        Ok(())
    }
}

/// One judged trial: where it started, how large it was, and what it decided.
struct Attempt {
    start: f64,
    step: f64,
    end: f64,
    decision: StepDecision,
    normalized_error: f64,
    suggested: f64,
}

/// One embedded trial from `(start, state)`: `(trial state, error estimate, end instant)`.
fn trial(
    start: f64,
    step_value: f64,
    state: [f64; 1],
    workspace: &mut StepWorkspace<f64, 7>,
) -> ([f64; 1], [f64; 1], f64) {
    let start = Instant::new(Time::from_base(start)).expect("invariant: finite start");
    let step = StepSize::new(Time::from_base(step_value)).expect("invariant: positive step");
    let mut primary = [0.0];
    let mut error = [0.0];
    let report = step_embedded_into(
        &GaussianDecay,
        DormandPrince,
        start,
        step,
        &state,
        EmbeddedOutputs::new(&mut primary, &mut error),
        workspace,
    )
    .expect("invariant: infallible system, matched dimensions");
    (primary, error, *report.end().as_time().as_base())
}

/// Everything a consumer holds after marching to `END_TIME`.
struct March {
    history: Vec<Attempt>,
    time: f64,
    state: [f64; 1],
    certified_tolerance: f64,
}

fn march() -> March {
    let controller =
        AdaptiveController::<f64>::new(ABSOLUTE_TOLERANCE, RELATIVE_TOLERANCE, 0.9, 0.2, 5.0)
            .expect("invariant: valid controller");
    let mut workspace = StepWorkspace::<f64, 7>::new(1).expect("invariant: nonzero workspace");

    let mut time = 0.0_f64;
    let mut state = [1.0_f64];
    let mut proposed = FIRST_STEP;
    let mut history: Vec<Attempt> = Vec::new();
    let mut certified_tolerance = 0.0_f64;

    while time < END_TIME {
        // Land on the end instant: `END_TIME - time` is exact for
        // `END_TIME / 2 <= time <= END_TIME` (Sterbenz), so the clipped step
        // ends on `END_TIME` with no rounding.
        let remaining = END_TIME - time;
        let step_value = if proposed < remaining {
            proposed
        } else {
            assert!(
                time >= END_TIME / 2.0,
                "clipping outside the exact-subtraction range: t = {time}"
            );
            remaining
        };
        let (primary, error, end) = trial(time, step_value, state, &mut workspace);
        let step = StepSize::new(Time::from_base(step_value)).expect("invariant: positive step");
        let report = controller
            .assess::<ORDER>(step, error[0], primary[0])
            .expect("invariant: finite observation");
        history.push(Attempt {
            start: time,
            step: step_value,
            end,
            decision: report.decision(),
            normalized_error: report.normalized_error(),
            suggested: *report.suggested_step().as_time().as_base(),
        });

        if report.decision() == StepDecision::Accept {
            time = end;
            state = primary;
            certified_tolerance += ABSOLUTE_TOLERANCE + RELATIVE_TOLERANCE * primary[0].abs();
        } else {
            assert_eq!(report.decision(), StepDecision::Reject);
            // The consumer-visible state is untouched: the retry starts
            // from the same instant and state, and the workspace the
            // rejected trial dirtied must not leak into it.
            let retry_value = *report.suggested_step().as_time().as_base();
            assert!(
                retry_value < step_value,
                "a rejected step of {step_value} was re-proposed as {retry_value}"
            );
            let mut fresh = StepWorkspace::<f64, 7>::new(1).expect("invariant: nonzero");
            let reused = trial(time, retry_value, state, &mut workspace);
            let isolated = trial(time, retry_value, state, &mut fresh);
            assert_eq!(
                (
                    reused.0[0].to_bits(),
                    reused.1[0].to_bits(),
                    reused.2.to_bits()
                ),
                (
                    isolated.0[0].to_bits(),
                    isolated.1[0].to_bits(),
                    isolated.2.to_bits()
                ),
                "a retry on the reused workspace differs from one on a fresh workspace"
            );
        }
        proposed = *report.suggested_step().as_time().as_base();
    }

    March {
        history,
        time,
        state,
        certified_tolerance,
    }
}

/// History: every attempt starts where the previous accepted one ended, so a
/// rejection never advances time; decisions agree with the normalized error;
/// the attempt after a rejection is the suggested step from the same instant.
fn assert_history_is_continuous(march: &March) {
    let mut committed = 0.0_f64;
    let mut rejections = 0_usize;
    let mut accepts = 0_usize;
    let mut rejected_after_accept = 0_usize;
    for (index, attempt) in march.history.iter().enumerate() {
        assert_eq!(attempt.start.to_bits(), committed.to_bits());
        if attempt.decision == StepDecision::Accept {
            assert!(attempt.normalized_error <= 1.0);
            committed = attempt.end;
            accepts += 1;
        } else {
            assert_eq!(attempt.decision, StepDecision::Reject);
            assert!(attempt.normalized_error > 1.0);
            assert!(attempt.suggested < attempt.step);
            rejections += 1;
            if accepts > 0 {
                rejected_after_accept += 1;
            }
            let next = march
                .history
                .get(index + 1)
                .expect("invariant: a rejection is retried");
            assert_eq!(next.step.to_bits(), attempt.suggested.to_bits());
            assert_eq!(next.start.to_bits(), attempt.start.to_bits());
        }
    }
    assert_eq!(march.time.to_bits(), END_TIME.to_bits());
    assert_eq!(committed.to_bits(), END_TIME.to_bits());
    assert!(
        rejections >= 1,
        "the first step must start outside tolerance"
    );
    assert!(
        rejected_after_accept >= 1,
        "no rejection followed an accepted step: the march never recovered mid-flight"
    );
    assert!(accepts >= 2, "a single accepted step is not a march");
}

fn assert_solution_is_certified(march: &March) {
    // Each accepted step has `|estimate| <= atol + rtol·|y|`. The
    // estimate is the embedded (lower-order) error, `O(h^5)`; the primary
    // result's own local error is `O(h^6)`, so `|primary - exact| <=
    // |estimate| + |embedded - exact|` with `|embedded - exact|` equal to the
    // estimate to leading order: twice the certified tolerance bounds each
    // local deviation. For `y' = -2ty` on `t >= 0` an earlier deviation is
    // multiplied by `e^(-(t_{k+1}² - t_k²)) <= 1` at every later step, so the
    // global deviation is at most the sum of the local ones.
    let exact = (-END_TIME * END_TIME).exp();
    let deviation = (march.state[0] - exact).abs();
    let bound = 2.0 * march.certified_tolerance;
    assert!(
        deviation <= bound,
        "the march ends at {}, {deviation:.3e} from the closed-form {exact}, \
         beyond the certified bound {bound:.3e}",
        march.state[0]
    );
    assert!(
        deviation > 0.0,
        "an exact result would mean no integration happened"
    );
}
#[test]
fn a_rejected_trial_leaves_the_march_where_it_was_and_the_retry_is_clean() {
    let march = march();
    assert_history_is_continuous(&march);
    assert_solution_is_certified(&march);
}
