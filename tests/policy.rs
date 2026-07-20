//! Validation and value semantics for time-orchestration policies.

use aequitas::systems::si::quantities::Time;
use horae::{
    adaptive::{AdaptiveController, AdaptiveError, AdaptiveParameter, StepDecision},
    events::{EventSchedule, ScheduleError},
    subcycling::{SubcycleError, SubcyclePlan},
    time::{Instant, StepSize, TimeError},
};

#[test]
fn time_values_enforce_finite_positive_contracts() {
    assert_eq!(
        Instant::new(Time::from_base(f64::NAN)),
        Err(TimeError::NonFinite)
    );
    assert_eq!(
        StepSize::new(Time::from_base(0.0)),
        Err(TimeError::NonPositiveStep)
    );
    assert_eq!(
        StepSize::new(Time::from_base(-1.0)),
        Err(TimeError::NonPositiveStep)
    );
    assert_eq!(
        StepSize::new(Time::from_base(f64::INFINITY)),
        Err(TimeError::NonFinite)
    );
}

#[test]
fn adaptive_controller_accepts_and_rejects_by_mixed_tolerance() {
    let controller = AdaptiveController::<f64>::new(1.0e-3, 1.0e-2, 0.9, 0.2, 5.0)
        .expect("invariant: valid controller");
    let step = StepSize::new(Time::from_base(0.1)).expect("invariant: positive fixture");

    let accepted = controller
        .assess::<4>(step, 0.005, 1.0)
        .expect("invariant: finite observation");
    assert_eq!(accepted.decision(), StepDecision::Accept);
    assert!((accepted.normalized_error() - 5.0 / 11.0).abs() <= 4.0 * f64::EPSILON);
    assert!(accepted.scale() > 1.0);

    let rejected = controller
        .assess::<4>(step, 0.022, 1.0)
        .expect("invariant: finite observation");
    assert_eq!(rejected.decision(), StepDecision::Reject);
    assert!((rejected.normalized_error() - 2.0).abs() <= 4.0 * f64::EPSILON);
    assert!(rejected.scale() < 1.0);
}

#[test]
fn adaptive_controller_reports_invalid_inputs() {
    assert_eq!(
        AdaptiveController::<f64>::new(0.0, 1.0e-2, 0.9, 0.2, 5.0),
        Err(AdaptiveError::InvalidParameter(
            AdaptiveParameter::AbsoluteTolerance
        ))
    );
    let controller = AdaptiveController::<f64>::new(1.0e-3, 1.0e-2, 0.9, 0.2, 5.0)
        .expect("invariant: valid controller");
    let step = StepSize::new(Time::from_base(0.1)).expect("invariant: positive fixture");
    assert_eq!(
        controller.assess::<0>(step, 0.0, 1.0),
        Err(AdaptiveError::InvalidOrder)
    );
    assert_eq!(
        controller.assess::<4>(step, f64::NAN, 1.0),
        Err(AdaptiveError::NonFiniteObservation)
    );
    let overflow_controller = AdaptiveController::<f64>::new(1.0, f64::MAX, 0.9, 0.2, 5.0)
        .expect("invariant: finite parameters satisfy construction");
    assert_eq!(
        overflow_controller.assess::<4>(step, 1.0, f64::MAX),
        Err(AdaptiveError::NonFiniteTolerance)
    );
}

#[test]
fn event_schedule_clips_to_exact_next_boundary() {
    let start: Instant<f64> =
        Instant::new(Time::from_base(1.0)).expect("invariant: finite fixture");
    let events = [
        start,
        Instant::new(Time::from_base(1.25)).expect("invariant: finite fixture"),
        Instant::new(Time::from_base(2.0)).expect("invariant: finite fixture"),
    ];
    let schedule = EventSchedule::new(&events).expect("invariant: sorted fixture");
    assert!(core::ptr::eq(schedule.events().as_ptr(), events.as_ptr()));

    let proposed = StepSize::new(Time::from_base(0.5)).expect("invariant: positive fixture");
    let clipped = schedule
        .clip_step(start, proposed)
        .expect("invariant: finite endpoint");
    assert_eq!(
        (*clipped.step().as_time().as_base()).to_bits(),
        0.25_f64.to_bits()
    );
    assert_eq!(clipped.event(), Some(events[1]));
    assert!(clipped.was_shortened());

    let exact = schedule
        .clip_step(
            start,
            StepSize::new(Time::from_base(0.25)).expect("invariant: positive fixture"),
        )
        .expect("invariant: finite endpoint");
    assert_eq!(exact.event(), Some(events[1]));
    assert!(!exact.was_shortened());
}

#[test]
fn event_schedule_rejects_decreasing_instants() {
    let events = [
        Instant::new(Time::from_base(2.0)).expect("invariant: finite fixture"),
        Instant::new(Time::from_base(1.0)).expect("invariant: finite fixture"),
    ];
    assert_eq!(
        EventSchedule::new(&events),
        Err(ScheduleError::Unsorted { index: 1 })
    );
}

#[test]
fn subcycle_plan_derives_only_ratio_and_child_step() {
    let plan = SubcyclePlan::<4>::new().expect("invariant: nonzero bounded ratio");
    let parent: StepSize<f64> =
        StepSize::new(Time::from_base(1.0)).expect("invariant: positive fixture");
    let child = plan
        .child_step(parent)
        .expect("invariant: representable child");
    assert_eq!(plan.ratio(), 4);
    assert_eq!((*child.as_time().as_base()).to_bits(), 0.25_f64.to_bits());
    assert_eq!(SubcyclePlan::<0>::new(), Err(SubcycleError::ZeroRatio));
}
