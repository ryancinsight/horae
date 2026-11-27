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
//!
//! The march is one generic function over the scalar. It runs at every
//! scalar the crate documents (ADR 0003): the native `f32` and `f64` and the
//! Eunomia `F32` and `F64` wrappers. Each run states its own tolerances; the
//! widths differ, so one tolerance cannot sit above every `EPSILON`.

use core::convert::Infallible;
use std::{
    any::type_name,
    panic::{AssertUnwindSafe, catch_unwind},
};

use aequitas::systems::si::quantities::Time;
use eunomia::{F32, F64, NumericElement, RealField};
use horae::{
    adaptive::{AdaptiveController, StepDecision},
    integration::{EmbeddedOutputs, StepWorkspace, step_embedded_into, tableau::DormandPrince},
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

const END_TIME: f64 = 3.0;
const FIRST_STEP: f64 = 0.5;
const SAFETY_FACTOR: f64 = 0.9;
const MINIMUM_SCALE: f64 = 0.2;
const MAXIMUM_SCALE: f64 = 5.0;
/// Rounded operations per result of one stage combination: a product and a
/// fused accumulation for each of the seven stages.
const COMBINATION_OPERATIONS: f64 = 14.0;
/// Rounded operations per stage that reach the result through the stage
/// derivative: up to six fused accumulations into the stage state, the fused
/// stage time, and the two products of `-2ty`. Each perturbs the derivative
/// by at most one unit roundoff of the derivative itself.
const STAGE_OPERATIONS: f64 = 9.0;
/// Rounded operations per result, the stage terms counted for all seven stages.
const ROUNDED_OPERATIONS: f64 = COMBINATION_OPERATIONS + 7.0 * STAGE_OPERATIONS;
/// Formal order of the Dormand--Prince primary result, as declared by the tableau.
const ORDER: usize = 5;

/// The per-scalar inputs of one march.
#[derive(Clone, Copy)]
struct Parameters {
    absolute_tolerance: f64,
    relative_tolerance: f64,
    /// Smallest positive normal value of the scalar, widened exactly.
    smallest_normal: f64,
}

#[derive(Clone, Copy)]
struct GaussianDecay;

/// `y' = -2ty`: the time argument enters, and the local error grows and then
/// shrinks along the march, so a step grown after an accept is later rejected.
impl<T: RealField> ExplicitSystem<T> for GaussianDecay {
    type Error = Infallible;

    fn evaluate(
        &self,
        time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        let time = *time.as_time().as_base();
        derivative[0] = -(time + time) * state[0];
        Ok(())
    }
}

/// The exact widening of a scalar's value, as a bit pattern.
///
/// Widening to `f64` is injective on non-NaN values and keeps the sign of
/// zero, so equal patterns mean equal values of the scalar.
fn bits<T: RealField>(value: T) -> u64 {
    value.to_f64().to_bits()
}

/// One judged trial: how large it was and what it decided.
struct Attempt<T> {
    step: T,
    decision: StepDecision,
    normalized_error: T,
    suggested: T,
}

/// One embedded trial from `(start, state)`: `(trial state, error estimate, end instant)`.
///
/// The trial must report the interval it was given: the start instant bitwise
/// and the end as the sum `start + step` in the scalar, computed here rather
/// than by `Instant::advance` so that a defect in `advance` is not shared by
/// both sides. `state` is `&[T]` inside the stepper, so a trial cannot alter it.
fn trial<T: RealField>(
    start_value: T,
    step_value: T,
    state: [T; 1],
    workspace: &mut StepWorkspace<T, 7>,
) -> ([T; 1], [T; 1], T) {
    let start = Instant::new(Time::from_base(start_value)).expect("invariant: finite start");
    let step = StepSize::new(Time::from_base(step_value)).expect("invariant: positive step");
    let mut primary = [<T as NumericElement>::ZERO];
    let mut error = [<T as NumericElement>::ZERO];
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
        bits(*report.start().as_time().as_base()),
        bits(start_value),
        "a trial from {start_value:?} reported another start instant"
    );
    assert_eq!(
        bits(end),
        bits(start_value + step_value),
        "a trial of size {step_value:?} from {start_value:?} reported end {end:?}"
    );
    (primary, error, end)
}

/// Longest run of attempts that leave `time` unchanged, so the march ends
/// even if a defect stops time from advancing.
///
/// A `Reject` has normalized error `e > 1`, so the suggested scale
/// `min(max(safety / e^(1/6), minimum_scale), maximum_scale)` is below
/// `safety`. Every attempted step is at most `END_TIME`, so `n`
/// consecutive rejections leave a proposal below `safety^n * END_TIME`
/// (up to a relative rounding of order `n * EPSILON`, far inside the slack of
/// the strict inequality). A step below the smallest normal value is never
/// rejected (hypothesis: its estimate is `O(h)` against the absolute
/// tolerance), so a stall is `n` rejections for the least `n` with that
/// product below the smallest normal value, plus the accept that advances
/// time. The bound is per stall, not on the total attempt count, which would
/// need the error constant of the pair.
fn stall_limit(safety: f64, smallest_normal: f64) -> usize {
    let mut step = END_TIME;
    let mut rejections = 0;
    while step >= smallest_normal {
        step *= safety;
        rejections += 1;
    }
    rejections + 1
}

/// Everything a consumer holds after marching to `END_TIME`.
struct March<T> {
    history: Vec<Attempt<T>>,
    time: T,
    state: [T; 1],
    deviation_bound: f64,
}

fn march<T: RealField>(parameters: Parameters) -> March<T> {
    let absolute = T::from_f64(parameters.absolute_tolerance);
    let relative = T::from_f64(parameters.relative_tolerance);
    let safety = T::from_f64(SAFETY_FACTOR);
    let controller = AdaptiveController::<T>::new(
        absolute,
        relative,
        safety,
        T::from_f64(MINIMUM_SCALE),
        T::from_f64(MAXIMUM_SCALE),
    )
    .expect("invariant: valid controller");
    let mut workspace = StepWorkspace::<T, 7>::new(1).expect("invariant: nonzero workspace");

    // The controller holds the scalar's roundings of the tolerances, and the
    // deviation bound uses those values rather than the decimal literals.
    let epsilon = T::EPSILON.to_f64();
    let (absolute, relative) = (absolute.to_f64(), relative.to_f64());
    // The estimate is the difference of two results that each carry up to
    // `ROUNDED_OPERATIONS` units of `epsilon / 2` at `|y_k| + |y_k+1| = 2|y|`,
    // so its roundoff is at most `2 * ROUNDED_OPERATIONS * epsilon * |y|`. A
    // decision driven by truncation needs a relative tolerance above it;
    // below it the decisions follow rounding, which this march does not model.
    let noise_floor = 2.0 * ROUNDED_OPERATIONS * epsilon;
    assert!(
        relative > noise_floor,
        "relative tolerance {relative:e} is not above the estimate roundoff floor {noise_floor:e}"
    );
    let stall_limit = stall_limit(safety.to_f64(), parameters.smallest_normal);

    let end_time = T::from_f64(END_TIME);
    let clip_floor = T::from_f64(END_TIME / 2.0);
    let mut time = <T as NumericElement>::ZERO;
    let mut state = [<T as NumericElement>::ONE];
    let mut proposed = T::from_f64(FIRST_STEP);
    let mut history: Vec<Attempt<T>> = Vec::new();
    let mut deviation_bound = 0.0_f64;
    let mut stalled = 0_usize;

    while time < end_time {
        stalled += 1;
        assert!(
            stalled <= stall_limit,
            "{stalled} attempts since time last advanced, at t = {time:?}"
        );
        // Land on the end instant: `END_TIME - time` is exact for
        // `END_TIME / 2 <= time <= END_TIME` (Sterbenz), so the clipped step
        // ends on `END_TIME` with no rounding.
        let remaining = end_time - time;
        let step_value = if proposed < remaining {
            proposed
        } else {
            assert!(
                time >= clip_floor,
                "the clipped step starts at t = {time:?}, below END_TIME / 2"
            );
            remaining
        };
        let (primary, error, end) = trial(time, step_value, state, &mut workspace);
        let step = StepSize::new(Time::from_base(step_value)).expect("invariant: positive step");
        let report = controller
            .assess::<ORDER>(step, error[0], primary[0])
            .expect("invariant: finite observation");
        // The controller's normalization from first principles: the estimate
        // over `atol + rtol |y|`, evaluated here in `f64` from the exactly
        // widened inputs. The controller rounds its fused tolerance and its
        // quotient once each (`EPSILON` in all) and the reference adds three
        // `f64` roundings, so the two agree within `3 EPSILON` relative.
        let expected = error[0].to_f64().abs() / (absolute + relative * primary[0].to_f64().abs());
        let reported = report.normalized_error().to_f64();
        assert!(
            (reported - expected).abs() <= 3.0 * epsilon * expected,
            "normalized error {reported:e} departs from {expected:e} for estimate {:?} at {:?}",
            error[0],
            primary[0]
        );
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
            let (previous, next) = (state[0].to_f64().abs(), primary[0].to_f64().abs());
            let tolerance = absolute + relative * next;
            let roundoff = ROUNDED_OPERATIONS * (epsilon / 2.0) * (previous + next);
            let end_value = end.to_f64();
            deviation_bound += (2.0 * tolerance * (1.0 + 2.0 * epsilon) + 5.0 * roundoff)
                * (end_value * end_value - END_TIME * END_TIME).exp();
            if end > time {
                stalled = 0;
            }
            time = end;
            state = primary;
        } else {
            assert_eq!(report.decision(), StepDecision::Reject);
            // The retry starts from the same `time` and `state`. `trial`
            // borrows the state as `&[T]`, so a rejected trial cannot
            // change it (the type guarantees this; no assertion tests it).
            // The mutable carry-over is the workspace the rejected trial
            // dirtied, which the comparison below tests.
            let retry_value = *report.suggested_step().as_time().as_base();
            assert!(
                retry_value < step_value,
                "a rejected step of {step_value:?} was re-proposed as {retry_value:?}"
            );
            let mut fresh = StepWorkspace::<T, 7>::new(1).expect("invariant: nonzero");
            let reused = trial(time, retry_value, state, &mut workspace);
            let isolated = trial(time, retry_value, state, &mut fresh);
            assert_eq!(
                (bits(reused.0[0]), bits(reused.1[0]), bits(reused.2)),
                (bits(isolated.0[0]), bits(isolated.1[0]), bits(isolated.2)),
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
fn assert_decisions_follow_the_error<T: RealField>(march: &March<T>) {
    let mut rejections = 0_usize;
    let mut accepts = 0_usize;
    let mut rejected_after_accept = 0_usize;
    for attempt in &march.history {
        if attempt.decision == StepDecision::Accept {
            assert!(attempt.normalized_error <= <T as NumericElement>::ONE);
            accepts += 1;
        } else {
            assert_eq!(attempt.decision, StepDecision::Reject);
            assert!(attempt.normalized_error > <T as NumericElement>::ONE);
            assert!(attempt.suggested < attempt.step);
            rejections += 1;
            if accepts > 0 {
                rejected_after_accept += 1;
            }
        }
    }
    assert_eq!(bits(march.time), END_TIME.to_bits());
    assert!(rejections >= 1, "the march recorded no rejection");
    assert!(
        rejected_after_accept >= 1,
        "no rejection followed an accepted step: the march never recovered mid-flight"
    );
    assert!(accepts >= 2, "a single accepted step is not a march");
}

fn assert_solution_is_certified<T: RealField>(march: &March<T>) {
    // With `delta_k` the local error of accepted step `k` from its computed
    // state, `y' = -2ty` on `t >= 0` gives the exact recursion
    // `e_{k+1} = P_k e_k + delta_k` with propagator
    // `P_k = e^(-(t_{k+1}² - t_k²)) <= 1`, so
    // `|e_N| <= sum_k |delta_k| e^(-(END_TIME² - t_{k+1}²))`.
    // The estimate `est` is primary minus embedded, `delta_p - delta_e`.
    // Hypothesis: `|delta_p| <= (2/3) |delta_e|` (the primary is the higher
    // order). Then `|delta_e| <= |est| + |delta_p|` gives `|delta_e| <= 3 |est|`
    // and `|delta_p| <= 2 |est|`. Each computed result differs from the
    // exact-arithmetic result of the same step by at most `rho`, so
    // `|est_exact| <= |est_computed| + 2 rho`, and an accepted step has
    // `|est_computed| <= (atol + rtol |y|)(1 + 2 EPSILON)`: the controller's
    // fused tolerance, its quotient and the subtraction each round once. The
    // step's deviation is `|delta_p| + rho <= 2 |est_computed| + 5 rho`.
    // `rho` is `ROUNDED_OPERATIONS` units of `EPSILON / 2` at
    // `|y_k| + |y_k+1|` (hypothesis: partial sums and stage states stay below
    // that scale, and each stage's weighted contribution `h |b_i| |k_i|` stays
    // below it), damped the same way. The bound is derived, and the damping
    // keeps it close enough to the deviation to expose a stage-coupling
    // defect.
    //
    // The state is widened to `f64` exactly, and the closed form is the `f64`
    // `exp`, within an `f64` ulp, which is far below the bound of every scalar.
    let exact = (-END_TIME * END_TIME).exp();
    let deviation = (march.state[0].to_f64() - exact).abs();
    let bound = march.deviation_bound;
    assert!(
        deviation <= bound,
        "the march ends at {:?}, {deviation:.3e} from the closed-form {exact}, \
         beyond the derived bound {bound:.3e}",
        march.state[0]
    );
    assert!(
        deviation > 0.0,
        "an exact result would mean no integration happened"
    );
}

/// One scalar's march: the generic march instantiated at it, and its inputs.
struct Instance {
    scalar: &'static str,
    run: fn(Parameters),
    parameters: Parameters,
}

fn instance<T: RealField>(parameters: Parameters) -> Instance {
    Instance {
        scalar: type_name::<T>(),
        run: |parameters| {
            let march = march::<T>(parameters);
            assert_decisions_follow_the_error(&march);
            assert_solution_is_certified(&march);
        },
        parameters,
    }
}

#[test]
fn a_rejected_trial_leaves_the_march_where_it_was_and_the_retry_is_clean() {
    // Every scalar runs and each failure names its scalar, so a defect that
    // one width hides is reported from the others rather than lost behind
    // the first panic.
    let single = Parameters {
        absolute_tolerance: 1.0e-6,
        relative_tolerance: 1.0e-4,
        smallest_normal: f64::from(f32::MIN_POSITIVE),
    };
    let double = Parameters {
        absolute_tolerance: 1.0e-10,
        relative_tolerance: 1.0e-8,
        smallest_normal: f64::MIN_POSITIVE,
    };
    let instances = [
        instance::<f32>(single),
        instance::<F32>(single),
        instance::<f64>(double),
        instance::<F64>(double),
    ];
    let failures: Vec<String> = instances
        .into_iter()
        .filter_map(
            |Instance {
                 scalar,
                 run,
                 parameters,
             }| {
                catch_unwind(AssertUnwindSafe(|| run(parameters)))
                    .err()
                    .map(|payload| {
                        let message = payload
                            .downcast_ref::<String>()
                            .map(String::as_str)
                            .or_else(|| payload.downcast_ref::<&str>().copied())
                            .unwrap_or("a non-string panic payload");
                        format!("{scalar}: {message}")
                    })
            },
        )
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
