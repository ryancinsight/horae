//! The iterate left by the last Newton correction is tested against the
//! residual tolerance like every earlier one.
//!
//! A budget of `n` corrections allows `n + 1` residual tests after the
//! predictor. The fixtures have a closed-form residual sequence, so the
//! tolerance sits between the residuals after the fifteenth and sixteenth
//! correction.

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

/// Residual `2^-18` after the sixteenth correction of the halving fixtures.
const RESIDUAL_AFTER_SIXTEENTH: f64 = 1.0 / 262_144.0;

/// Residual `2^-17` after the fifteenth correction of the halving fixtures.
const RESIDUAL_AFTER_FIFTEENTH: f64 = 1.0 / 131_072.0;

/// Tolerance of the board oracle, `5e-6`: above `2^-18 = 3.81e-6`, so the
/// sixteenth correction satisfies it, and below `2^-17 = 7.63e-6`, so the
/// fifteenth does not.
const BOARD_TOLERANCE: f64 = 5.0e-6;

/// Tolerance `2^-19`, half the residual after the sixteenth correction.
const TOLERANCE_BELOW_SIXTEENTH: f64 = 1.0 / 524_288.0;

/// Squared-residual fixture: residual `2^-34` after the fifteenth correction.
const SQUARE_RESIDUAL_AFTER_FIFTEENTH: f64 = 1.0 / 17_179_869_184.0;

/// Squared-residual fixture: residual `2^-36` after the sixteenth correction.
const SQUARE_RESIDUAL_AFTER_SIXTEENTH: f64 = 1.0 / 68_719_476_736.0;

/// Squared-residual fixture: `2^-37`, below the residual after the sixteenth.
const SQUARE_TOLERANCE_BELOW_SIXTEENTH: f64 = 1.0 / 137_438_953_472.0;

/// Calls made on a fixture.
#[derive(Default)]
struct Calls {
    evaluations: Cell<usize>,
    jacobians: Cell<usize>,
}

/// Decay `y' = -r·y` whose Jacobian is reported as zero, so the Newton matrix
/// is the identity and the iteration is `y <- y - g(y)`.
///
/// With `a = h·r` the residual is `g = (1 + a)·y - y_n` and the iteration
/// multiplies the error by `-a` per correction. For `a = 1/2` the residual
/// after `k` corrections from the predictor `(1 - a)·y_n` is `2^-(k + 2)`
/// when `y_n = 1`.
struct ZeroJacobianDecay<T> {
    rate: T,
    calls: Calls,
}

impl<T: RealField> ExplicitSystem<T> for ZeroJacobianDecay<T> {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        self.calls.evaluations.set(self.calls.evaluations.get() + 1);
        derivative[0] = -self.rate * state[0];
        Ok(())
    }
}

impl<T: RealField> ImplicitSystem<T> for ZeroJacobianDecay<T> {
    fn jacobian(
        &self,
        _time: Instant<T>,
        _state: &[T],
        jacobian: &mut [T],
    ) -> Result<(), Self::Error> {
        self.calls.jacobians.set(self.calls.jacobians.get() + 1);
        jacobian[0] = T::ZERO;
        Ok(())
    }
}

/// `f(y) = y - 1/2 - y²` with its exact Jacobian `1 - 2y`.
///
/// For `h = 1` and `y_n = 1/2` the residual is `g(y) = y²` and the Newton
/// matrix is `2y`, so each correction halves the iterate: the predictor is
/// `1/4` and after `k` corrections `y = 2^-(k + 2)` and `g = 2^-(2k + 4)`.
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
        self.calls.evaluations.set(self.calls.evaluations.get() + 1);
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
        self.calls.jacobians.set(self.calls.jacobians.get() + 1);
        jacobian[0] = T::ONE - T::from_f64(2.0) * state[0];
        Ok(())
    }
}

/// How a step ends and what it cost.
struct Outcome<T> {
    converged: bool,
    output: T,
    evaluations: usize,
    jacobians: usize,
}

fn run<T, System>(
    system: &System,
    calls: &Calls,
    duration: T,
    initial: T,
    tolerance: T,
) -> Outcome<T>
where
    T: RealField,
    System: ImplicitSystem<T, Error = Infallible>,
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
    let evaluations = calls.evaluations.get();
    let converged = match result {
        Ok(report) => {
            assert_eq!(report.evaluations(), evaluations);
            true
        }
        Err(ImplicitStepError::Solve(SolveError::NonConvergence)) => false,
        Err(other) => panic!("unexpected failure {other:?}"),
    };
    Outcome {
        converged,
        output: output[0],
        evaluations,
        jacobians: calls.jacobians.get(),
    }
}

/// Step `y' = -r·y` with `a = h·r = 1/2` and `y_n = 1`.
///
/// `h = 2^-7` and `r = 64` make every product and sum of the iteration exact:
/// the iterate after `k` corrections is a dyadic rational with `k + 1`
/// fractional bits, within every shipped mantissa for `k <= 16`.
fn step_halving_decay<T: RealField>(tolerance: f64) -> Outcome<T> {
    let system = ZeroJacobianDecay {
        rate: T::from_f64(64.0),
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
    // The sixteenth correction meets the board tolerance: accepted there,
    // with the residual test that follows it.
    for tolerance in [BOARD_TOLERANCE, RESIDUAL_AFTER_SIXTEENTH] {
        let outcome = step_halving_decay::<T>(tolerance);
        assert!(outcome.converged, "tolerance {tolerance}");
        assert_eq!(outcome.output, halving_iterate::<T>(CORRECTION_BUDGET));
        assert_eq!(outcome.evaluations, evaluations_after(CORRECTION_BUDGET));
        assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
    }

    // The fifteenth correction already meets its residual: the step stops
    // there and makes no sixteenth correction.
    let outcome = step_halving_decay::<T>(RESIDUAL_AFTER_FIFTEENTH);
    assert!(outcome.converged);
    assert_eq!(outcome.output, halving_iterate::<T>(CORRECTION_BUDGET - 1));
    assert_eq!(
        outcome.evaluations,
        evaluations_after(CORRECTION_BUDGET - 1)
    );
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET - 1);

    // No tolerance reachable within the budget: all corrections are made, the
    // last is tested, and the step still fails; no seventeenth is made.
    let outcome = step_halving_decay::<T>(TOLERANCE_BELOW_SIXTEENTH);
    assert!(!outcome.converged);
    assert_eq!(outcome.evaluations, evaluations_after(CORRECTION_BUDGET));
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
}

fn step_squared_residual<T: RealField>(tolerance: f64) -> Outcome<T> {
    let system = SquaredResidual {
        calls: Calls::default(),
    };
    run(
        &system,
        &system.calls,
        T::ONE,
        T::from_f64(0.5),
        T::from_f64(tolerance),
    )
}

/// The squared-residual fixture needs `y²` exact down to `2^-36`, which a
/// 53-bit mantissa holds and a 24-bit one does not, so it runs at `f64` only.
fn verify_squared_residual<T: RealField>() {
    let sixteenth = T::from_f64(1.0 / 262_144.0);
    let outcome = step_squared_residual::<T>(SQUARE_RESIDUAL_AFTER_SIXTEENTH);
    assert!(outcome.converged);
    assert_eq!(outcome.output, sixteenth);
    assert_eq!(outcome.evaluations, evaluations_after(CORRECTION_BUDGET));
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET);

    let outcome = step_squared_residual::<T>(SQUARE_RESIDUAL_AFTER_FIFTEENTH);
    assert!(outcome.converged);
    assert_eq!(outcome.output, T::from_f64(1.0 / 131_072.0));
    assert_eq!(
        outcome.evaluations,
        evaluations_after(CORRECTION_BUDGET - 1)
    );
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET - 1);

    let outcome = step_squared_residual::<T>(SQUARE_TOLERANCE_BELOW_SIXTEENTH);
    assert!(!outcome.converged);
    assert_eq!(outcome.evaluations, evaluations_after(CORRECTION_BUDGET));
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
}

#[test]
fn the_last_correction_is_tested_against_the_tolerance() {
    verify_halving_decay::<f32>();
    verify_halving_decay::<f64>();
    verify_halving_decay::<eunomia::F32>();
    verify_halving_decay::<eunomia::F64>();
}

#[test]
fn the_last_correction_of_the_board_decay_is_tested() {
    // `h = 0.01`, `r = 50`: `a = 1/2` as in the halving fixture but with
    // inexact products. The residual `2^-18` after the sixteenth correction
    // differs from the tolerance by more than `1e-6`, many orders above the
    // rounding of its evaluation at `f64`.
    let system = ZeroJacobianDecay {
        rate: 50.0_f64,
        calls: Calls::default(),
    };
    let outcome = run(&system, &system.calls, 0.01, 1.0, BOARD_TOLERANCE);
    assert!(outcome.converged);
    assert_eq!(outcome.evaluations, evaluations_after(CORRECTION_BUDGET));
    assert_eq!(outcome.jacobians, CORRECTION_BUDGET);
}

#[test]
fn the_last_correction_of_a_nonlinear_step_is_tested() {
    verify_squared_residual::<f64>();
    verify_squared_residual::<eunomia::F64>();
}
