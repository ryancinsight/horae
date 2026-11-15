//! The explicit-Euler predictor seeds Newton and is never the accepted step,
//! and a correction is accepted only above its own rounding floor.
//!
//! For `y' = -r·y` with `a = h·r` the predictor is `(1 - a)·y_n` and its
//! residual is `-a²·y_n`, which an absolute tolerance accepts for a small
//! `y_n` although `|1 - a| > 1` grows the decaying mode. The accepted step
//! must be the backward-Euler value `y_n / (1 + a)`. Once `a` exceeds
//! `1/ε` the correction itself cancels a predictor of magnitude `a·|y_n|`
//! and its rounding error exceeds `|y_n|`, so a further correction from the
//! now small iterate is required.

use core::{cell::Cell, convert::Infallible};

use aequitas::systems::si::quantities::Time;
use eunomia::RealField;
use horae::{
    integration::{BackwardEuler, ImplicitWorkspace, step_implicit_into},
    system::{ExplicitSystem, ImplicitSystem},
    time::{Instant, StepSize},
};

/// Independent decays `y_i' = -r_i·y_i` with exact diagonal Jacobian.
struct Diagonal<T, const N: usize> {
    rates: [T; N],
    evaluations: Cell<usize>,
}

impl<T: RealField, const N: usize> ExplicitSystem<T> for Diagonal<T, N> {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        self.evaluations.set(self.evaluations.get() + 1);
        for (slope, (rate, value)) in derivative.iter_mut().zip(self.rates.iter().zip(state)) {
            *slope = -*rate * *value;
        }
        Ok(())
    }
}

impl<T: RealField, const N: usize> ImplicitSystem<T> for Diagonal<T, N> {
    fn jacobian(
        &self,
        _time: Instant<T>,
        _state: &[T],
        jacobian: &mut [T],
    ) -> Result<(), Self::Error> {
        jacobian.fill(T::ZERO);
        for (i, rate) in self.rates.iter().enumerate() {
            jacobian[i * N + i] = -*rate;
        }
        Ok(())
    }
}

/// Rounding of one Newton correction, in units of `ε·(|x| + |y_n|)`.
///
/// With `u = ε/2` and start `x`, the residual `g` carries
/// `u·((3a + 2)|x| + 2|y_n|)` rounding, so the correction `-g/(1 + a)` carries
/// at most `u·(3|x| + 2|y_n|)` plus `3u·|Δ|` from the matrix and division, and
/// the final sum `x + Δ` adds `u·|y|`. The total is below
/// `u·(6|x| + 6|y_n|) = 3ε(|x| + |y_n|)`; the factor 4 also covers the
/// rounding of the reference value `y_n / (1 + a)`. The start cancels
/// against the correction, so the error scales with `|x|`, not with `|y|`.
const ROUNDING_UNITS: f64 = 4.0;

/// Error bound after `corrections` Newton corrections from the predictor.
///
/// The first correction starts at `p = (1 - a)·y_n`; each later correction
/// starts at the previous iterate, whose magnitude is at most the exact
/// value `|y_n| / (1 + a)` plus its error bound.
fn accepted_error_bound<T: RealField>(a: T, initial: T, corrections: usize) -> T {
    let units = T::from_f64(ROUNDING_UNITS) * T::EPSILON;
    let exact = (initial / (T::ONE + a)).abs();
    let mut start = (T::ONE - a).abs() * initial.abs();
    let mut bound = units * (start + initial.abs());
    for _ in 1..corrections {
        start = exact + bound;
        bound = units * (start + initial.abs());
    }
    bound
}

/// Run one step of `N` independent decays and check every component.
///
/// `corrections` is the number of Newton corrections the acceptance rule
/// requires for this stiffness; each costs one residual evaluation beyond the
/// predictor and the first residual.
fn verify_backward_euler_step<T: RealField, const N: usize>(
    rates: [T; N],
    state: [T; N],
    corrections: usize,
) {
    let arrange = tracing::info_span!("arrange").entered();
    let system = Diagonal {
        rates,
        evaluations: Cell::new(0),
    };
    let start = Instant::new(Time::from_base(T::ZERO)).expect("finite origin");
    let duration = T::from_f64(0.01);
    let step = StepSize::new(Time::from_base(duration)).expect("positive finite duration");
    let mut output = [T::ZERO; N];
    let mut workspace = ImplicitWorkspace::new(N).expect("nonzero dimension");
    let tolerance = T::from_f64(1e-6);
    drop(arrange);

    let act = tracing::info_span!("act").entered();
    let report = step_implicit_into(
        &system,
        BackwardEuler,
        start,
        step,
        &state,
        &mut output,
        &mut workspace,
        tolerance,
    )
    .expect("a linear system converges");
    drop(act);

    let _assert = tracing::info_span!("assert").entered();
    for ((rate, initial), accepted) in system.rates.iter().zip(&state).zip(&output) {
        let a = duration * *rate;
        let expected = *initial / (T::ONE + a);
        let bound = accepted_error_bound(a, *initial, corrections);
        assert!(
            accepted.abs() <= initial.abs(),
            "decaying mode grew from {initial:?} to {accepted:?}"
        );
        assert!(
            (*accepted - expected).abs() <= bound,
            "accepted {accepted:?} differs from backward Euler {expected:?} by more than {bound:?}"
        );
    }
    // Predictor, residual at the predictor, then one residual per correction.
    assert_eq!(
        report.evaluations(),
        corrections + 2,
        "rates {:?}, states {state:?}, output {output:?}",
        system.rates
    );
    assert_eq!(system.evaluations.get(), corrections + 2);
}

fn verify_scalar<T: RealField>() {
    let value = T::from_f64;
    // The board case: a = 100, y_n = 1e-12, predictor residual 1e-8 < tolerance.
    verify_backward_euler_step::<T, 1>([value(1e4)], [value(1e-12)], 1);
    // A quiescent slow component beside a tiny fast one: a state-norm-scaled
    // tolerance still accepts the predictor of the fast component.
    verify_backward_euler_step::<T, 2>([value(0.0), value(1e4)], [T::ONE, value(1e-12)], 1);
    // A zero state is a fixed point: the mandatory correction is zero.
    verify_backward_euler_step::<T, 2>([value(1e4), T::ONE], [value(1e-12), T::ZERO], 1);
}

/// Stiffness `a = h·r` at multiples of `1/ε`, with `h = 0.01`.
///
/// The first correction cancels a predictor of magnitude `a·|y_n|`, so its
/// rounding error is `ε·a·|y_n|` and exceeds `|y_n|` once `a > 1/ε`. The
/// acceptance rule requires `6ε·(|Δ| + |y| + |y_n|) <= max(|y_n|, |y|)`, which
/// the first correction meets at `a·ε = 1/32` (`6ε·a = 0.19`) and fails from
/// `a·ε = 1` (`6ε·a = 6`) onward, since `|y| <= 4·|y_n|` there; the second
/// correction starts at a magnitude `ε·a·|y_n|` and meets it up to
/// `a·ε = 2^10`.
///
/// The states are `s·ε` for `s` in `1e-8..1e-9`, so that the residual left by
/// one correction, `ε·a²·|y_n|`, stays below the tolerance `1e-6` at
/// `a·ε = 1/32` and the residual test does not add a correction.
fn verify_stiff_scalar<T: RealField>() {
    let step = T::from_f64(0.01);
    for (multiple, corrections) in [(1.0 / 32.0, 1), (1.0, 2), (8.0, 2), (1024.0, 2)] {
        let rate = T::from_f64(multiple) / (T::EPSILON * step);
        for scale in [1e-8, 3e-9, 1e-9] {
            let initial = T::from_f64(scale) * T::EPSILON;
            verify_backward_euler_step::<T, 1>([rate], [initial], corrections);
        }
        let tiny = T::from_f64(1e-9) * T::EPSILON;
        verify_backward_euler_step::<T, 2>([T::ZERO, rate], [T::ONE, tiny], corrections);
    }
}

#[test]
fn stiff_decay_is_never_accepted_at_the_predictor() {
    verify_scalar::<f32>();
    verify_scalar::<f64>();
    verify_scalar::<eunomia::F32>();
    verify_scalar::<eunomia::F64>();
}

#[test]
fn stiff_decay_does_not_grow_past_the_correction_rounding_floor() {
    verify_stiff_scalar::<f32>();
    verify_stiff_scalar::<f64>();
    verify_stiff_scalar::<eunomia::F32>();
    verify_stiff_scalar::<eunomia::F64>();
}
