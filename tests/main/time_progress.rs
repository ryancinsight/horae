//! Positive durations must advance the represented clock before state work.

use core::{cell::Cell, convert::Infallible};

use aequitas::systems::si::quantities::Time;
use eunomia::RealField;
use horae::{
    events::EventSchedule,
    integration::{
        BackwardEuler, EmbeddedOutputs, ImplicitWorkspace, SliceRole, StepError, StepWorkspace,
        step_embedded_into, step_implicit_into, step_into,
        tableau::{DormandPrince, Euler, ExplicitTableau, Midpoint, Rk4},
    },
    system::{ExplicitSystem, ImplicitSystem},
    time::{Instant, StepSize, TimeError},
};

const STALLED: &str = "positive step does not advance simulation time";

fn traced(test: impl FnOnce()) {
    let subscriber = tracing_subscriber::fmt()
        .with_test_writer()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .finish();
    tracing::subscriber::with_default(subscriber, test);
}

fn instant<T: RealField>(value: T) -> Instant<T> {
    Instant::new(Time::from_base(value)).expect("finite analytical fixture")
}

fn step<T: RealField>(value: T) -> StepSize<T> {
    StepSize::new(Time::from_base(value)).expect("positive analytical fixture")
}

fn start<T: RealField>() -> Instant<T> {
    // At 2/epsilon = 2^p, upward neighbors are two apart. Adding one is
    // exactly the midpoint and rounds back to the even start.
    instant((T::ONE + T::ONE) / T::EPSILON)
}

fn verify_instants<T: RealField>() {
    let arrange = tracing::info_span!("arrange").entered();
    let one = T::ONE;
    let two = one + one;
    let half = one / two;
    let origin = start::<T>();
    let value = *origin.as_time().as_base();
    drop(arrange);
    let act = tracing::info_span!("act").entered();
    let results = [
        origin.advance(step(one)),
        origin.advance(step(half)),
        instant(-value).advance(step(half)),
    ];
    drop(act);
    let _assert = tracing::info_span!("assert").entered();
    for result in results {
        assert_eq!(
            result.expect_err("clock must progress").to_string(),
            STALLED
        );
    }
    assert_eq!(origin.advance(step(two)), Ok(instant(value + two)));
    assert_eq!(
        instant(-value).advance(step(one)),
        Ok(instant(-value + one))
    );
    assert_eq!(instant(-one).advance(step(one)), Ok(instant(T::ZERO)));
    assert_eq!(
        instant(T::ZERO).advance(step(T::EPSILON)),
        Ok(instant(T::EPSILON))
    );
    // A midpoint next to an odd significand rounds upward and stays valid.
    assert_eq!(
        instant(value + two).advance(step(one)),
        Ok(instant(value + two + two))
    );
}

#[test]
fn advance_requires_strict_progress_for_native_and_wrapped_scalars() {
    traced(|| {
        verify_instants::<f32>();
        verify_instants::<f64>();
        verify_instants::<eunomia::F32>();
        verify_instants::<eunomia::F64>();
    });
}

/// The real ODE y' = 1, with zero Jacobian and observable evaluations.
#[derive(Default)]
struct UnitRate(Cell<usize>);

impl<T: RealField> ExplicitSystem<T> for UnitRate {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<T>,
        _state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error> {
        self.0.set(self.0.get() + 1);
        derivative.fill(T::ONE);
        Ok(())
    }
}

impl<T: RealField> ImplicitSystem<T> for UnitRate {
    fn jacobian(
        &self,
        _time: Instant<T>,
        _state: &[T],
        jacobian: &mut [T],
    ) -> Result<(), Self::Error> {
        jacobian.fill(T::ZERO);
        Ok(())
    }
}

fn verify_fixed<T, Method, const STAGES: usize>()
where
    T: RealField,
    Method: ExplicitTableau<STAGES> + Default,
{
    let arrange = tracing::info_span!("arrange", stages = STAGES).entered();
    let system = UnitRate::default();
    let mut workspace = StepWorkspace::<T, STAGES>::new(1).expect("one component");
    let mut output = [T::ONE];
    drop(arrange);
    let act = tracing::info_span!("act").entered();
    let result = step_into(
        &system,
        Method::default(),
        start(),
        step(T::ONE),
        &[T::ONE],
        &mut output,
        &mut workspace,
    );
    drop(act);
    let _assert = tracing::info_span!("assert").entered();
    assert_eq!(
        result.expect_err("stalled step must fail").to_string(),
        format!("step time is invalid: {STALLED}")
    );
    assert_eq!(system.0.get(), 0);
    assert_eq!(output, [T::ONE]);
}

fn verify_integrators<T: RealField>() {
    verify_fixed::<T, Euler, 1>();
    verify_fixed::<T, Midpoint, 2>();
    verify_fixed::<T, Rk4, 4>();
    verify_fixed::<T, DormandPrince, 7>();
    let arrange = tracing::info_span!("arrange", method = "embedded and implicit").entered();
    let system = UnitRate::default();
    let mut workspace = StepWorkspace::<T, 7>::new(1).expect("one component");
    let mut implicit = ImplicitWorkspace::<T>::new(1).expect("one component");
    let mut output = [T::ONE];
    let mut error = [T::ONE];
    drop(arrange);
    let act = tracing::info_span!("act").entered();
    let embedded = step_embedded_into(
        &system,
        DormandPrince,
        start(),
        step(T::ONE),
        &[T::ONE],
        EmbeddedOutputs::new(&mut output, &mut error),
        &mut workspace,
    );
    let backward = step_implicit_into(
        &system,
        BackwardEuler,
        start(),
        step(T::ONE),
        &[T::ONE],
        &mut output,
        &mut implicit,
        T::EPSILON,
    );
    drop(act);
    let _assert = tracing::info_span!("assert").entered();
    assert_eq!(
        embedded
            .expect_err("stalled embedded step must fail")
            .to_string(),
        format!("step time is invalid: {STALLED}")
    );
    assert_eq!(
        backward
            .expect_err("stalled implicit step must fail")
            .to_string(),
        format!("step time is invalid: {STALLED}")
    );
    assert_eq!(system.0.get(), 0);
    assert_eq!(output, [T::ONE]);
    assert_eq!(error, [T::ONE]);
}

#[test]
fn integration_rejects_stalled_time_before_evaluation_or_output() {
    traced(|| {
        verify_integrators::<f32>();
        verify_integrators::<f64>();
        verify_integrators::<eunomia::F32>();
        verify_integrators::<eunomia::F64>();
    });
}

fn verify_schedule<T: RealField>() {
    let arrange = tracing::info_span!("arrange").entered();
    let origin = start::<T>();
    let events = [instant(*origin.as_time().as_base() + (T::ONE + T::ONE))];
    let schedule = EventSchedule::new(&events).expect("sorted singleton");
    drop(arrange);
    let act = tracing::info_span!("act").entered();
    let result = schedule.clip_step(origin, step(T::ONE));
    drop(act);
    let _assert = tracing::info_span!("assert").entered();
    assert_eq!(
        result
            .expect_err("stalled clock cannot reach future event")
            .to_string(),
        STALLED
    );
    let clipped = schedule
        .clip_step(origin, step(T::ONE + T::ONE))
        .expect("neighbor is reachable");
    assert_eq!(clipped.event(), Some(events[0]));
    assert_eq!(origin.advance(clipped.step()), Ok(events[0]));
    assert!(!clipped.was_shortened());
}

#[test]
fn event_clipping_rejects_stalled_proposals_and_retains_reachable_events() {
    traced(|| {
        verify_schedule::<f32>();
        verify_schedule::<f64>();
        verify_schedule::<eunomia::F32>();
        verify_schedule::<eunomia::F64>();
    });
}

#[test]
fn dimensions_keep_precedence_over_stalled_time() {
    traced(|| {
        let arrange = tracing::info_span!("arrange").entered();
        let system = UnitRate::default();
        let mut workspace = StepWorkspace::<f64, 1>::new(1).expect("one component");
        drop(arrange);
        let act = tracing::info_span!("act").entered();
        let result = step_into(
            &system,
            Euler,
            start(),
            step(1.0),
            &[1.0],
            &mut [],
            &mut workspace,
        );
        drop(act);
        let _assert = tracing::info_span!("assert").entered();
        assert_eq!(
            result,
            Err(StepError::DimensionMismatch {
                role: SliceRole::Output,
                expected: 1,
                actual: 0
            })
        );
        assert_eq!(system.0.get(), 0);
    });
}

#[test]
fn subnormal_advancement_and_overflow_keep_their_exact_boundaries() {
    traced(|| {
        let arrange = tracing::info_span!("arrange").entered();
        let tiny = f64::from_bits(1);
        let maximum = instant(f64::MAX);
        drop(arrange);
        let act = tracing::info_span!("act").entered();
        let subnormal = instant(0.0).advance(step(tiny));
        let overflow = maximum.advance(step(f64::MAX));
        drop(act);
        let _assert = tracing::info_span!("assert").entered();
        assert_eq!(subnormal, Ok(instant(tiny)));
        assert_eq!(overflow, Err(TimeError::NonFinite));
        assert_eq!(instant(-f64::MAX).advance(step(f64::MAX)), Ok(instant(0.0)));
    });
}
