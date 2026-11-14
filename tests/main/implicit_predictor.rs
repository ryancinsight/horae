//! The explicit-Euler predictor seeds Newton and is never the accepted step.
//!
//! For `y' = -r·y` with `a = h·r` the predictor is `(1 - a)·y_n` and its
//! residual is `-a²·y_n`, which an absolute tolerance accepts for a small
//! `y_n` although `|1 - a| > 1` grows the decaying mode. The accepted step
//! must be the backward-Euler value `y_n / (1 + a)`.

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

/// Accepted-step rounding bound in units of `ε·(|1 - a| + 1)·|y_n|`.
///
/// With `u = ε/2` and predictor `p = (1 - a)·y_n`, the residual `g` carries
/// `u·((3a + 2)|p| + 2|y_n|)` rounding, so the correction `-g/(1 + a)` carries
/// at most `u·(3|p| + 2|y_n|)` plus `3u·|Δ|` from the matrix and division, and
/// the final sum `p + Δ` adds `u·|y|`. The total is below
/// `u·(6|p| + 6|y_n|) = 3ε·(|1 - a| + 1)·|y_n|`; the factor 4 also covers the
/// rounding of the reference value `y_n / (1 + a)`. The predictor cancels
/// against the correction, so the error scales with `|p|`, not with `|y|`.
const ROUNDING_UNITS: f64 = 4.0;

fn verify_backward_euler_step<T: RealField, const N: usize>(rates: [f64; N], state: [f64; N]) {
    let arrange = tracing::info_span!("arrange").entered();
    let system = Diagonal {
        rates: rates.map(T::from_f64),
        evaluations: Cell::new(0),
    };
    let start = Instant::new(Time::from_base(T::ZERO)).expect("finite origin");
    let duration = T::from_f64(0.01);
    let step = StepSize::new(Time::from_base(duration)).expect("positive finite duration");
    let state = state.map(T::from_f64);
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
        let bound = T::from_f64(ROUNDING_UNITS)
            * T::EPSILON
            * ((T::ONE - a).abs() + T::ONE)
            * initial.abs();
        assert!(
            accepted.abs() <= initial.abs(),
            "decaying mode grew from {initial:?} to {accepted:?}"
        );
        assert!(
            (*accepted - expected).abs() <= bound,
            "accepted {accepted:?} differs from backward Euler {expected:?} by more than {bound:?}"
        );
    }
    // Predictor, residual at the predictor, then the residual after the one
    // Newton correction that is exact on a linear system.
    assert_eq!(report.evaluations(), 3);
    assert_eq!(system.evaluations.get(), 3);
}

fn verify_scalar<T: RealField>() {
    // The board case: a = 100, y_n = 1e-12, predictor residual 1e-8 < tolerance.
    verify_backward_euler_step::<T, 1>([1e4], [1e-12]);
    // A quiescent slow component beside a tiny fast one: a state-norm-scaled
    // tolerance still accepts the predictor of the fast component.
    verify_backward_euler_step::<T, 2>([0.0, 1e4], [1.0, 1e-12]);
    // A zero state is a fixed point: the mandatory correction is zero.
    verify_backward_euler_step::<T, 2>([1e4, 1.0], [1e-12, 0.0]);
}

#[test]
fn stiff_decay_is_never_accepted_at_the_predictor() {
    verify_scalar::<f32>();
    verify_scalar::<f64>();
    verify_scalar::<eunomia::F32>();
    verify_scalar::<eunomia::F64>();
}
