//! A successful implicit solve requires every residual component to be finite.

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

fn traced(test: impl FnOnce()) {
    let subscriber = tracing_subscriber::fmt()
        .with_test_writer()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .finish();
    tracing::subscriber::with_default(subscriber, test);
}

/// Independent decays `y_i' = -r_i y_i` with exact diagonal Jacobian.
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

fn start<T: RealField>() -> Instant<T> {
    Instant::new(Time::from_base(T::ZERO)).expect("finite origin")
}

fn step<T: RealField>(duration: T) -> StepSize<T> {
    StepSize::new(Time::from_base(duration)).expect("positive finite duration")
}

fn verify_finite_overflow<T: RealField>(maximum: T) {
    let arrange = tracing::info_span!("arrange").entered();
    let system = Diagonal {
        rates: [T::ONE],
        evaluations: Cell::new(0),
    };
    let two = T::ONE + T::ONE;
    let mut workspace = ImplicitWorkspace::new(1).expect("one component");
    let mut output = [T::ZERO];
    assert!(maximum.is_finite());
    assert!((maximum / (two + T::ONE)).is_finite());
    drop(arrange);
    let act = tracing::info_span!("act").entered();
    let result = step_implicit_into(
        &system,
        BackwardEuler,
        start(),
        step(two),
        &[maximum],
        &mut output,
        &mut workspace,
        T::from_f64(1e-8),
    );
    drop(act);
    let _assert = tracing::info_span!("assert").entered();
    assert_eq!(
        result,
        Err(ImplicitStepError::Solve(SolveError::NonConvergence))
    );
    // FMA predictor is -MAX; the residual (-MAX)-MAX-2*MAX overflows.
    // No Newton correction can legitimately certify that residual.
    assert_eq!(output, [-maximum]);
    assert_eq!(system.evaluations.get(), 2);
}

#[test]
fn finite_decay_overflow_cannot_certify_convergence() {
    traced(|| {
        verify_finite_overflow(f32::MAX);
        verify_finite_overflow(f64::MAX);
        verify_finite_overflow(eunomia::F32(f32::MAX));
        verify_finite_overflow(eunomia::F64(f64::MAX));
    });
}

fn verify_mixed_components<T: RealField>() {
    let _fixture = tracing::info_span!("mixed_components").entered();
    for invalid in [T::NAN, T::INFINITY, T::ZERO - T::INFINITY] {
        for position in 0..2 {
            let arrange = tracing::info_span!("arrange", position).entered();
            let system = Diagonal {
                rates: [T::ONE; 2],
                evaluations: Cell::new(0),
            };
            let mut state = [T::ZERO; 2];
            state[position] = invalid;
            let mut output = [T::ONE; 2];
            let mut workspace = ImplicitWorkspace::new(2).expect("two components");
            drop(arrange);
            let act = tracing::info_span!("act").entered();
            let result = step_implicit_into(
                &system,
                BackwardEuler,
                start(),
                step(T::ONE),
                &state,
                &mut output,
                &mut workspace,
                T::EPSILON,
            );
            drop(act);
            let _assert = tracing::info_span!("assert").entered();
            assert_eq!(
                result,
                Err(ImplicitStepError::Solve(SolveError::NonConvergence))
            );
            assert!(!output[position].is_finite());
            assert_eq!(output[1 - position], T::ZERO);
            assert_eq!(system.evaluations.get(), 2);
        }
    }
}

#[test]
fn nonfinite_components_are_not_hidden_by_finite_components() {
    traced(|| {
        verify_mixed_components::<f32>();
        verify_mixed_components::<f64>();
        verify_mixed_components::<eunomia::F32>();
        verify_mixed_components::<eunomia::F64>();
    });
}

/// Conservative exchange `y' = k * [y_1-y_0, y_0-y_1]`.
struct Exchange<T> {
    rate: T,
    evaluations: Cell<usize>,
}

impl<T: RealField> ExplicitSystem<T> for Exchange<T> {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        self.evaluations.set(self.evaluations.get() + 1);
        derivative[0] = self.rate * (state[1] - state[0]);
        derivative[1] = self.rate * (state[0] - state[1]);
        Ok(())
    }
}

impl<T: RealField> ImplicitSystem<T> for Exchange<T> {
    fn jacobian(
        &self,
        _time: Instant<T>,
        _state: &[T],
        jacobian: &mut [T],
    ) -> Result<(), Self::Error> {
        jacobian.copy_from_slice(&[-self.rate, self.rate, self.rate, -self.rate]);
        Ok(())
    }
}

fn verify_matrix_overflow<T: RealField>(maximum: T, smallest: T) {
    let arrange = tracing::info_span!("arrange").entered();
    let system = Exchange {
        rate: maximum,
        evaluations: Cell::new(0),
    };
    let mut workspace = ImplicitWorkspace::new(2).expect("two components");
    let mut output = [T::ZERO; 2];
    let two = T::ONE + T::ONE;
    assert!(maximum.is_finite() && smallest.is_finite() && smallest > T::ZERO);
    drop(arrange);
    let act = tracing::info_span!("act").entered();
    let result = step_implicit_into(
        &system,
        BackwardEuler,
        start(),
        step(two),
        &[smallest, T::ZERO],
        &mut output,
        &mut workspace,
        T::EPSILON,
    );
    drop(act);
    let _assert = tracing::info_span!("assert").entered();
    assert_eq!(
        result,
        Err(ImplicitStepError::Solve(SolveError::NonConvergence))
    );
    // The predictor and first residual are finite. I - h*J overflows;
    // elimination computes inf/inf and the correction becomes NaN.
    assert!(output.iter().all(|value| value.is_nan()));
    assert_eq!(system.evaluations.get(), 3);
}

#[test]
fn overflowing_newton_matrix_cannot_certify_nan_corrections() {
    traced(|| {
        verify_matrix_overflow(f32::MAX, f32::from_bits(1));
        verify_matrix_overflow(f64::MAX, f64::from_bits(1));
        verify_matrix_overflow(eunomia::F32(f32::MAX), eunomia::F32(f32::from_bits(1)));
        verify_matrix_overflow(eunomia::F64(f64::MAX), eunomia::F64(f64::from_bits(1)));
    });
}

fn verify_reuse<T: RealField>() {
    let arrange = tracing::info_span!("arrange").entered();
    let two = T::ONE + T::ONE;
    let four = two + two;
    let eight = four + four;
    let half = T::ONE / two;
    let system = Diagonal {
        rates: [two, four + two],
        evaluations: Cell::new(0),
    };
    let mut workspace = ImplicitWorkspace::new(2).expect("two components");
    let mut output = [T::ZERO; 2];
    drop(arrange);
    let act = tracing::info_span!("act").entered();
    let failed = step_implicit_into(
        &system,
        BackwardEuler,
        start(),
        step(half),
        &[T::NAN, T::ZERO],
        &mut output,
        &mut workspace,
        T::EPSILON,
    );
    let accepted = step_implicit_into(
        &system,
        BackwardEuler,
        start(),
        step(half),
        &[four, eight],
        &mut output,
        &mut workspace,
        T::EPSILON,
    );
    drop(act);
    let _assert = tracing::info_span!("assert").entered();
    assert_eq!(
        failed,
        Err(ImplicitStepError::Solve(SolveError::NonConvergence))
    );
    let report = accepted.expect("exact diagonal backward-Euler solution");
    // [4/(1 + .5*2), 8/(1 + .5*6)] = [2, 2], all dyadic operations.
    assert_eq!(output, [two; 2]);
    assert_eq!(report.evaluations(), 3);
    assert_eq!(*report.end().as_time().as_base(), half);
    assert_eq!(system.evaluations.get(), 5);
}

#[test]
fn finite_solution_and_workspace_reuse_preserve_computed_values() {
    traced(|| {
        verify_reuse::<f32>();
        verify_reuse::<f64>();
        verify_reuse::<eunomia::F32>();
        verify_reuse::<eunomia::F64>();
    });
}
