//! The iterate left by the last Newton correction is tested against the
//! residual tolerance like every earlier one.
//!
//! A budget of `n` corrections allows `n + 1` residual tests after the
//! predictor. The fixtures have closed-form residuals, so the tolerance, the
//! finiteness guard, the rounding floor and a failing system are each
//! exercised at the test that follows the last correction.

use core::{cell::Cell, convert::Infallible};

use aequitas::systems::si::quantities::Time;
use eunomia::RealField;
use horae::{
    integration::{
        BackwardEuler, ImplicitStepError, ImplicitWorkspace, SolveError, step_implicit_into,
    },
    system::{ExplicitSystem, ImplicitSystem},
    time::{Instant, StepSize},
};

/// Newton corrections one step may make.
const CORRECTION_BUDGET: usize = 16;

/// Residual evaluations of a step that makes `corrections` corrections: the
/// predictor's derivative, then one residual test before each correction and
/// one after the last.
const fn evaluations_after(corrections: usize) -> usize {
    corrections + 2
}

/// The halving decay's residual after the sixteenth correction, `2^-18`.
const RESIDUAL_AFTER_SIXTEENTH: f64 = 1.0 / 262_144.0;

/// The halving decay's residual after the fifteenth correction, `2^-17`.
const RESIDUAL_AFTER_FIFTEENTH: f64 = 1.0 / 131_072.0;

/// Tolerance `5e-6` of the board oracle, between `2^-18 = 3.81e-6` and
/// `2^-17 = 7.63e-6`.
const BOARD_TOLERANCE: f64 = 5.0e-6;

/// Tolerance `2^-19`, half the residual after the sixteenth correction.
const TOLERANCE_BELOW_SIXTEENTH: f64 = 1.0 / 524_288.0;

/// The squared-residual fixture's residual `2^-34` after the fifteenth
/// correction.
const SQUARE_RESIDUAL_AFTER_FIFTEENTH: f64 = 1.0 / 17_179_869_184.0;

/// The squared-residual fixture's residual `2^-36` after the sixteenth
/// correction.
const SQUARE_RESIDUAL_AFTER_SIXTEENTH: f64 = 1.0 / 68_719_476_736.0;

/// Tolerance `2^-37`, half the squared-residual fixture's residual after the
/// sixteenth correction.
const SQUARE_TOLERANCE_BELOW_SIXTEENTH: f64 = 1.0 / 137_438_953_472.0;

/// Corrections after which the squared-residual fixture at `f32` has residual
/// exactly `0`: see [`SquaredResidual`].
const SQUARE_ROUNDED_CORRECTIONS: usize = 11;

/// Bound `16ε` on the board decay's output error against the exact iteration
/// of `a = 1/2`. Each correction rounds by at most
/// `3ε·(|Δ| + |y| + |y_n|) <= 7.5ε` (the library's own bound, with
/// `|Δ| <= 1/2`, `|y| <= 1`, `|y_n| = 1`) and later corrections contract it
/// by `a = 1/2`, so the sum is at most `15ε`. Rounding `0.01` moves `a` by at
/// most `a·ε/2 = ε/4`, and `|dy_16/da| <= sum j·a^(j-1) <= 4`, so that adds
/// at most `ε`.
const BOARD_OUTPUT_UNITS: f64 = 16.0;

/// Tolerance of the cancelling-kick fixture: above the residual `1` of each
/// cancelling correction, below the residual `2^(p + 1) - 1` of each kick
/// (`p >= 23` mantissa bits).
const CANCELLING_TOLERANCE: f64 = 4.0;

/// Calls made on a fixture.
#[derive(Default)]
struct Calls {
    evaluations: Cell<usize>,
    jacobians: Cell<usize>,
}

impl Calls {
    fn count_evaluation(&self) -> usize {
        self.evaluations.set(self.evaluations.get() + 1);
        self.evaluations.get()
    }

    fn count_jacobian(&self) -> usize {
        self.jacobians.set(self.jacobians.get() + 1);
        self.jacobians.get()
    }
}

/// Failure injected by a fixture.
#[derive(Debug, PartialEq)]
struct Injected;

/// What the fixture reports as its Jacobian.
///
/// `Zero` makes the Newton matrix the identity, so for the decay `y' = -r·y`
/// with `a = h·r` the residual is `g = (1 + a)·y - y_n` and the iteration
/// `y <- y - g(y)` multiplies the error by `-a` per correction. For `a = 1/2`
/// and `y_n = 1` the residual after `k` corrections from the predictor
/// `(1 - a)·y_n` is `2^-(k + 2)`.
///
/// `Alternating` takes `h = 1`, `r = 1`, `y_n = 1`: the residual is
/// `g(y) = 2y - 1` and the predictor `0`. Let `ε = 2^-p` be the machine
/// epsilon of `T`. Odd Jacobian calls report `1 - ε`: the Newton matrix is
/// `1 - (1 - ε) = ε`, the increment `1/ε = 2^p`, the iterate `2^p` and its
/// residual `2^(p + 1) - 1`, which has `p + 1` bits. Even calls report
/// `-(1 - ε)`: the Newton matrix is `2 - ε = 2·(1 - 2^-(p + 1))`, the
/// increment `-(2^(p + 1) - 1)/(2 - ε) = -2^p` exactly, the iterate returns to
/// `0` and the residual to `-1`. The rounding floor of an even correction is
/// `6ε·(2^p + 0 + 1) = 6 + 6ε`, above the scale `max(|y_n|, |y|) = 1`, so it
/// is never accepted, though its residual `1` is within the tolerance. The
/// sixteenth correction is even; the backward-Euler value is `1/2`.
enum JacobianReport {
    Zero,
    Alternating,
}

/// Decay `y' = -r·y`; `evaluate` returns NaN on call `nan_on_call` and fails
/// on call `fail_on_call`.
struct Decay<T> {
    rate: T,
    jacobian: JacobianReport,
    nan_on_call: Option<usize>,
    fail_on_call: Option<usize>,
    calls: Calls,
}

impl<T: RealField> ExplicitSystem<T> for Decay<T> {
    type Error = Injected;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        let call = self.calls.count_evaluation();
        if self.fail_on_call == Some(call) {
            return Err(Injected);
        }
        derivative[0] = if self.nan_on_call == Some(call) {
            T::NAN
        } else {
            -self.rate * state[0]
        };
        Ok(())
    }
}

impl<T: RealField> ImplicitSystem<T> for Decay<T> {
    fn jacobian(
        &self,
        _time: Instant<T>,
        _state: &[T],
        jacobian: &mut [T],
    ) -> Result<(), Self::Error> {
        let call = self.calls.count_jacobian();
        let kick = T::ONE - T::EPSILON;
        jacobian[0] = match self.jacobian {
            JacobianReport::Zero => T::ZERO,
            JacobianReport::Alternating if call % 2 == 1 => kick,
            JacobianReport::Alternating => -kick,
        };
        Ok(())
    }
}

struct Outcome<T, E> {
    /// The step's report evaluation count, or its failure.
    result: Result<usize, ImplicitStepError<E>>,
    output: T,
    evaluations: usize,
    jacobians: usize,
}

fn non_convergence<E>() -> Result<usize, ImplicitStepError<E>> {
    Err(ImplicitStepError::Solve(SolveError::NonConvergence))
}

fn run<T, System>(
    system: &System,
    calls: &Calls,
    duration: T,
    initial: T,
    tolerance: T,
) -> Outcome<T, System::Error>
where
    T: RealField,
    System: ImplicitSystem<T>,
{
    let arrange = tracing::info_span!("arrange").entered();
    let start = Instant::new(Time::from_base(T::ZERO)).expect("finite origin");
    let step = StepSize::new(Time::from_base(duration)).expect("positive finite duration");
    let mut output = [T::ZERO];
    let mut workspace = ImplicitWorkspace::new(1).expect("nonzero dimension");
    drop(arrange);

    let act = tracing::info_span!("act").entered();
    let result = step_implicit_into(
        system,
        BackwardEuler,
        start,
        step,
        &[initial],
        &mut output,
        &mut workspace,
        tolerance,
    );
    drop(act);

    let _assert = tracing::info_span!("assert").entered();
    Outcome {
        result: result.map(|report| report.evaluations()),
        output: output[0],
        evaluations: calls.evaluations.get(),
        jacobians: calls.jacobians.get(),
    }
}

/// Step `y' = -r·y` with `a = h·r = 1/2` and `y_n = 1`.
///
/// `h = 2^-7` and `r = 64` make every product and sum of the iteration exact:
/// the iterate after `k` corrections is a dyadic rational with `k + 1`
/// fractional bits, within every shipped mantissa for `k <= 16`.
fn step_halving_decay<T: RealField>(
    tolerance: f64,
    nan_on_call: Option<usize>,
    fail_on_call: Option<usize>,
) -> Outcome<T, Injected> {
    let system = Decay {
        rate: T::from_f64(64.0),
        jacobian: JacobianReport::Zero,
        nan_on_call,
        fail_on_call,
        calls: Calls::default(),
    };
    run(
        &system,
        &system.calls,
        T::from_f64(1.0 / 128.0),
        T::ONE,
        T::from_f64(tolerance),
    )
}

/// Iterate after `corrections` corrections of `y <- 1 - y/2` from `1/2`.
fn halving_iterate<T: RealField>(corrections: usize) -> T {
    let half = T::from_f64(0.5);
    let mut iterate = half;
    for _ in 0..corrections {
        iterate = T::ONE - half * iterate;
    }
    iterate
}

fn verify_halving_decay<T: RealField>() {
    // Accepted at the first residual test that meets the tolerance: after
    // the sixteenth correction for the board tolerance and `2^-18`, after
    // the fifteenth for `2^-17`, with no further correction.
    for (tolerance, corrections) in [
        (BOARD_TOLERANCE, CORRECTION_BUDGET),
        (RESIDUAL_AFTER_SIXTEENTH, CORRECTION_BUDGET),
        (RESIDUAL_AFTER_FIFTEENTH, CORRECTION_BUDGET - 1),
    ] {
        let outcome = step_halving_decay::<T>(tolerance, None, None);
        let evaluations = evaluations_after(corrections);
        assert_eq!(outcome.result, Ok(evaluations), "tolerance {tolerance}");
        assert_eq!(outcome.output, halving_iterate::<T>(corrections));
        assert_eq!(outcome.evaluations, evaluations);
        assert_eq!(outcome.jacobians, corrections);
    }

    // No tolerance reachable within the budget: all corrections are made, the
    // last is tested, and the step still fails; no seventeenth is made.
    let outcome = step_halving_decay::<T>(TOLERANCE_BELOW_SIXTEENTH, None, None);
    assert_eq!(outcome.result, non_convergence());
    assert_eq!(outcome.evaluations, evaluations_after(CORRECTION_BUDGET));
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
}

/// A NaN derivative in the test after the last correction fails the step
/// although the tolerance would accept the finite iterate: an unguarded
/// maximum discards NaN and leaves a zero norm.
fn verify_nonfinite_last_residual<T: RealField>() {
    let last_test = evaluations_after(CORRECTION_BUDGET);
    for tolerance in [RESIDUAL_AFTER_SIXTEENTH, TOLERANCE_BELOW_SIXTEENTH] {
        let outcome = step_halving_decay::<T>(tolerance, Some(last_test), None);
        assert_eq!(outcome.result, non_convergence(), "tolerance {tolerance}");
        assert_eq!(outcome.evaluations, last_test);
        assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
    }
}

/// An iterate below its rounding floor is not accepted at the last test
/// either, whatever its residual.
fn verify_unresolved_last_correction<T: RealField>() {
    let system = Decay {
        rate: T::ONE,
        jacobian: JacobianReport::Alternating,
        nan_on_call: None,
        fail_on_call: None,
        calls: Calls::default(),
    };
    let tolerance = T::from_f64(CANCELLING_TOLERANCE);
    let outcome = run(&system, &system.calls, T::ONE, T::ONE, tolerance);
    assert_eq!(outcome.result, non_convergence());
    assert_eq!(outcome.output, T::ZERO);
    assert_eq!(outcome.evaluations, evaluations_after(CORRECTION_BUDGET));
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
}

/// A system that fails on the evaluation after the last correction fails the
/// step with its own error, not `NonConvergence`.
fn verify_failure_after_the_last_correction<T: RealField>() {
    let last_test = evaluations_after(CORRECTION_BUDGET);
    for tolerance in [RESIDUAL_AFTER_SIXTEENTH, TOLERANCE_BELOW_SIXTEENTH] {
        let outcome = step_halving_decay::<T>(tolerance, None, Some(last_test));
        assert_eq!(
            outcome.result,
            Err(ImplicitStepError::System(Injected)),
            "tolerance {tolerance}"
        );
        assert_eq!(outcome.evaluations, last_test);
        assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
    }
}

/// `f(y) = y - 1/2 - y²` with its exact Jacobian `1 - 2y`.
///
/// For `h = 1` and `y_n = 1/2` the residual is `g(y) = y²` and the Newton
/// matrix is `2y`, so each correction halves the iterate: the predictor is
/// `1/4` and after `k` corrections `y = 2^-(k + 2)` and `g = 2^-(2k + 4)`.
/// `y²` itself is exact in every shipped scalar, but the derivative
/// `y - 1/2 - y²` spans `2^-1` down to `2^(-2k - 4)`, which a mantissa of
/// `p` bits holds only while `2k + 4 <= p`; `f64` (`p = 53`) holds all
/// sixteen corrections. At `f32` (`p = 24`) the sum first rounds at
/// `y = 2^-13`, where `y² = 2^-26` is exactly half the spacing `2^-25` of
/// `y - 1/2` and the tie rounds to the even neighbour `y - 1/2`, so the
/// derivative equals it and the residual is exactly `0`: the step is
/// accepted after `k = 11` corrections whatever the tolerance.
struct SquaredResidual {
    calls: Calls,
}

impl<T: RealField> ExplicitSystem<T> for SquaredResidual {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        self.calls.count_evaluation();
        derivative[0] = state[0] - T::from_f64(0.5) - state[0] * state[0];
        Ok(())
    }
}

impl<T: RealField> ImplicitSystem<T> for SquaredResidual {
    fn jacobian(
        &self,
        _time: Instant<T>,
        state: &[T],
        jacobian: &mut [T],
    ) -> Result<(), Self::Error> {
        self.calls.count_jacobian();
        jacobian[0] = T::ONE - T::from_f64(2.0) * state[0];
        Ok(())
    }
}

fn step_squared_residual<T: RealField>(tolerance: f64) -> Outcome<T, Infallible> {
    let system = SquaredResidual {
        calls: Calls::default(),
    };
    let half = T::from_f64(0.5);
    run(&system, &system.calls, T::ONE, half, T::from_f64(tolerance))
}

/// Iterate `2^-(corrections + 2)` of the squared-residual fixture.
fn squared_iterate<T: RealField>(corrections: usize) -> T {
    let mut iterate = T::from_f64(0.25);
    for _ in 0..corrections {
        iterate = iterate * T::from_f64(0.5);
    }
    iterate
}

/// A mantissa of at least 36 bits (`2k + 4` with `k = 16`) holds the whole residual sequence.
fn verify_squared_residual<T: RealField>() {
    for (tolerance, corrections) in [
        (SQUARE_RESIDUAL_AFTER_SIXTEENTH, CORRECTION_BUDGET),
        (SQUARE_RESIDUAL_AFTER_FIFTEENTH, CORRECTION_BUDGET - 1),
    ] {
        let outcome = step_squared_residual::<T>(tolerance);
        let evaluations = evaluations_after(corrections);
        assert_eq!(outcome.result, Ok(evaluations), "tolerance {tolerance}");
        assert_eq!(outcome.output, squared_iterate::<T>(corrections));
        assert_eq!(outcome.evaluations, evaluations);
        assert_eq!(outcome.jacobians, corrections);
    }
    let outcome = step_squared_residual::<T>(SQUARE_TOLERANCE_BELOW_SIXTEENTH);
    assert_eq!(outcome.result, non_convergence());
    assert_eq!(outcome.evaluations, evaluations_after(CORRECTION_BUDGET));
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
}

/// A 24-bit mantissa makes the residual exactly `0` after
/// [`SQUARE_ROUNDED_CORRECTIONS`] corrections, below every tolerance.
fn verify_rounded_squared_residual<T: RealField>() {
    for tolerance in [
        SQUARE_RESIDUAL_AFTER_FIFTEENTH,
        SQUARE_RESIDUAL_AFTER_SIXTEENTH,
        SQUARE_TOLERANCE_BELOW_SIXTEENTH,
    ] {
        let outcome = step_squared_residual::<T>(tolerance);
        let evaluations = evaluations_after(SQUARE_ROUNDED_CORRECTIONS);
        assert_eq!(outcome.result, Ok(evaluations), "tolerance {tolerance}");
        assert_eq!(
            outcome.output,
            squared_iterate::<T>(SQUARE_ROUNDED_CORRECTIONS)
        );
        assert_eq!(outcome.evaluations, evaluations);
        assert_eq!(outcome.jacobians, SQUARE_ROUNDED_CORRECTIONS);
    }
}

#[test]
fn the_last_correction_is_tested_against_the_tolerance() {
    verify_halving_decay::<f32>();
    verify_halving_decay::<f64>();
    verify_halving_decay::<eunomia::F32>();
    verify_halving_decay::<eunomia::F64>();
}

#[test]
fn a_nonfinite_residual_after_the_last_correction_fails_the_step() {
    verify_nonfinite_last_residual::<f32>();
    verify_nonfinite_last_residual::<f64>();
    verify_nonfinite_last_residual::<eunomia::F32>();
    verify_nonfinite_last_residual::<eunomia::F64>();
}

#[test]
fn the_last_correction_is_accepted_only_above_its_rounding_floor() {
    verify_unresolved_last_correction::<f32>();
    verify_unresolved_last_correction::<f64>();
    verify_unresolved_last_correction::<eunomia::F32>();
    verify_unresolved_last_correction::<eunomia::F64>();
}

#[test]
fn a_failure_after_the_last_correction_is_the_system_error() {
    verify_failure_after_the_last_correction::<f32>();
    verify_failure_after_the_last_correction::<f64>();
    verify_failure_after_the_last_correction::<eunomia::F32>();
    verify_failure_after_the_last_correction::<eunomia::F64>();
}

#[test]
fn the_last_correction_of_the_board_decay_is_tested() {
    // `h = 0.01`, `r = 50`: `a = 1/2` as in the halving decay, with inexact
    // products. The output stays within `BOARD_OUTPUT_UNITS` of the exact
    // iteration, and the residual `2^-18` differs from the tolerance by more
    // than `1e-6`, orders above its rounding at `f64`.
    let system = Decay {
        rate: 50.0_f64,
        jacobian: JacobianReport::Zero,
        nan_on_call: None,
        fail_on_call: None,
        calls: Calls::default(),
    };
    let outcome = run(&system, &system.calls, 0.01, 1.0, BOARD_TOLERANCE);
    assert_eq!(outcome.result, Ok(evaluations_after(CORRECTION_BUDGET)));
    let error = (outcome.output - halving_iterate::<f64>(CORRECTION_BUDGET)).abs();
    assert!(
        error <= BOARD_OUTPUT_UNITS * f64::EPSILON,
        "output {} differs by {error}",
        outcome.output
    );
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
}

#[test]
fn the_last_correction_of_a_nonlinear_step_is_tested() {
    verify_squared_residual::<f64>();
    verify_squared_residual::<eunomia::F64>();
    verify_rounded_squared_residual::<f32>();
    verify_rounded_squared_residual::<eunomia::F32>();
}
