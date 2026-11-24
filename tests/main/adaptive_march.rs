//! A reject-then-accept march with a recorded step history.
//!
//! `adaptive_loop.rs` drives one rejection loop from `t = 0` and stops at the
//! first accepted step. A consumer marches: it keeps `(time, state)` while a
//! trial is judged, discards a rejected trial, retries from the same instant
//! with the suggested step, and commits only an accepted one. This test is
//! that consumer, on `y' = -2ty` with the closed form `y(t) = y0·e^(-t²)`, and
//! it asserts what the caller can observe: every trial reports the interval it
//! was given, a retry on a reused workspace equals one on a fresh workspace,
//! and the march ends on the end instant within a derived deviation bound.

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
const SAFETY_FACTOR: f64 = 0.9;
const MINIMUM_SCALE: f64 = 0.2;
const MAXIMUM_SCALE: f64 = 5.0;
/// Rounded operations per result of one stage combination: a product and a
/// fused accumulation for each of the seven stages.
const ROUNDED_OPERATIONS: f64 = 14.0;
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

/// One judged trial: how large it was and what it decided.
struct Attempt {
    step: f64,
    decision: StepDecision,
    normalized_error: f64,
    suggested: f64,
}

/// One embedded trial from `(start, state)`: `(trial state, error estimate, end instant)`.
///
/// The trial must report the interval it was given: the start instant bitwise
/// and the end as the f64 sum `start + step`, computed here rather than by
/// `Instant::advance` so that a defect in `advance` is not shared by both
/// sides. `state` is `&[f64]` inside the stepper, so a trial cannot alter it.
fn trial(
    start_value: f64,
    step_value: f64,
    state: [f64; 1],
    workspace: &mut StepWorkspace<f64, 7>,
) -> ([f64; 1], [f64; 1], f64) {
    let start = Instant::new(Time::from_base(start_value)).expect("invariant: finite start");
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
    let end = *report.end().as_time().as_base();
    assert_eq!(
        report.start().as_time().as_base().to_bits(),
        start_value.to_bits(),
        "a trial from {start_value} reported another start instant"
    );
    assert_eq!(
        end.to_bits(),
        (start_value + step_value).to_bits(),
        "a trial of size {step_value} from {start_value} reported end {end}"
    );
    (primary, error, end)
}

/// Longest run of attempts that leave `time` unchanged, so the march ends
/// even if a defect stops time from advancing.
///
/// A `Reject` has normalized error `e > 1`, so the suggested scale
/// `min(max(SAFETY_FACTOR / e^(1/6), MINIMUM_SCALE), MAXIMUM_SCALE)` is below
/// `SAFETY_FACTOR`. Every attempted step is at most `END_TIME`, so `n`
/// consecutive rejections leave a proposal below `SAFETY_FACTOR^n * END_TIME`
/// (up to a relative rounding of order `n * EPSILON`, far inside the slack of
/// the strict inequality). A step below the smallest normal `f64` is never
/// rejected (hypothesis: its estimate is `O(h)` against `ABSOLUTE_TOLERANCE`),
/// so a stall is `n` rejections for the least `n` with that product below
/// `f64::MIN_POSITIVE`, plus the accept that advances time. The bound is per
/// stall, not on the total attempt count, which would need the error constant
/// of the pair.
const STALL_LIMIT: usize = {
    let mut step = END_TIME;
    let mut rejections = 0;
    while step >= f64::MIN_POSITIVE {
        step *= SAFETY_FACTOR;
        rejections += 1;
    }
    rejections + 1
};

/// Everything a consumer holds after marching to `END_TIME`.
struct March {
    history: Vec<Attempt>,
    time: f64,
    state: [f64; 1],
    deviation_bound: f64,
}

fn march() -> March {
    let controller = AdaptiveController::<f64>::new(
        ABSOLUTE_TOLERANCE,
        RELATIVE_TOLERANCE,
        SAFETY_FACTOR,
        MINIMUM_SCALE,
        MAXIMUM_SCALE,
    )
    .expect("invariant: valid controller");
    let mut workspace = StepWorkspace::<f64, 7>::new(1).expect("invariant: nonzero workspace");

    let mut time = 0.0_f64;
    let mut state = [1.0_f64];
    let mut proposed = FIRST_STEP;
    let mut history: Vec<Attempt> = Vec::new();
    let mut deviation_bound = 0.0_f64;
    let mut stalled = 0_usize;

    while time < END_TIME {
        stalled += 1;
        assert!(
            stalled <= STALL_LIMIT,
            "{stalled} attempts since time last advanced, at t = {time}"
        );
        // Land on the end instant: `END_TIME - time` is exact for
        // `END_TIME / 2 <= time <= END_TIME` (Sterbenz), so the clipped step
        // ends on `END_TIME` with no rounding.
        let remaining = END_TIME - time;
        let step_value = if proposed < remaining {
            proposed
        } else {
            assert!(
                time >= END_TIME / 2.0,
                "the clipped step starts at t = {time}, below END_TIME / 2"
            );
            remaining
        };
        let (primary, error, end) = trial(time, step_value, state, &mut workspace);
        let step = StepSize::new(Time::from_base(step_value)).expect("invariant: positive step");
        let report = controller
            .assess::<ORDER>(step, error[0], primary[0])
            .expect("invariant: finite observation");
        history.push(Attempt {
            step: step_value,
            decision: report.decision(),
            normalized_error: report.normalized_error(),
            suggested: *report.suggested_step().as_time().as_base(),
        });

        if report.decision() == StepDecision::Accept {
            // This step's local deviation, damped to `END_TIME` by the exact
            // propagator `e^(-(END_TIME² - end²))`; see
            // `assert_solution_is_certified`.
            let tolerance = ABSOLUTE_TOLERANCE + RELATIVE_TOLERANCE * primary[0].abs();
            let roundoff =
                ROUNDED_OPERATIONS * (f64::EPSILON / 2.0) * (state[0].abs() + primary[0].abs());
            deviation_bound +=
                (2.0 * tolerance + roundoff) * (end * end - END_TIME * END_TIME).exp();
            if end > time {
                stalled = 0;
            }
            time = end;
            state = primary;
        } else {
            assert_eq!(report.decision(), StepDecision::Reject);
            // The retry starts from the same `time` and `state`. `trial`
            // borrows the state as `&[f64]`, so a rejected trial cannot
            // change it (the type guarantees this; no assertion tests it).
            // The mutable carry-over is the workspace the rejected trial
            // dirtied, which the comparison below tests.
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
        deviation_bound,
    }
}

/// Decisions agree with the normalized error, a rejection suggests a smaller
/// step, and the march covers both a rejection after an accept and the end
/// instant.
fn assert_decisions_follow_the_error(march: &March) {
    let mut rejections = 0_usize;
    let mut accepts = 0_usize;
    let mut rejected_after_accept = 0_usize;
    for attempt in &march.history {
        if attempt.decision == StepDecision::Accept {
            assert!(attempt.normalized_error <= 1.0);
            accepts += 1;
        } else {
            assert_eq!(attempt.decision, StepDecision::Reject);
            assert!(attempt.normalized_error > 1.0);
            assert!(attempt.suggested < attempt.step);
            rejections += 1;
            if accepts > 0 {
                rejected_after_accept += 1;
            }
        }
    }
    assert_eq!(march.time.to_bits(), END_TIME.to_bits());
    assert!(rejections >= 1, "the march recorded no rejection");
    assert!(
        rejected_after_accept >= 1,
        "no rejection followed an accepted step: the march never recovered mid-flight"
    );
    assert!(accepts >= 2, "a single accepted step is not a march");
}

fn assert_solution_is_certified(march: &March) {
    // With `delta_k` the local error of accepted step `k` from its computed
    // state, `y' = -2ty` on `t >= 0` gives the exact recursion
    // `e_{k+1} = P_k e_k + delta_k` with propagator
    // `P_k = e^(-(t_{k+1}² - t_k²)) <= 1`, so
    // `|e_N| <= sum_k |delta_k| e^(-(END_TIME² - t_{k+1}²))`.
    // The estimate `est` is primary minus embedded, `delta_p - delta_e`.
    // Hypothesis: `|delta_p| <= (2/3) |delta_e|` (the primary is the higher
    // order). Then `|delta_e| <= |est| + |delta_p|` gives `|delta_e| <= 3 |est|`
    // and `|delta_p| <= 2 |est| <= 2 (atol + rtol |y|)` at an accepted step.
    // The recorded bound adds the rounding of the 14 operations of the stage
    // combination, each at most half an `EPSILON` of a partial sum
    // (hypothesis: partial sums stay below `|y_k| + |y_{k+1}|`), damped the
    // same way. The bound is derived, and the damping keeps it close enough
    // to the deviation to expose a stage-coupling defect.
    let exact = (-END_TIME * END_TIME).exp();
    let deviation = (march.state[0] - exact).abs();
    let bound = march.deviation_bound;
    assert!(
        deviation <= bound,
        "the march ends at {}, {deviation:.3e} from the closed-form {exact}, \
         beyond the derived bound {bound:.3e}",
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
    assert_decisions_follow_the_error(&march);
    assert_solution_is_certified(&march);
}
