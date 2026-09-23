//! Property tests for integration and exact event clipping.

use core::convert::Infallible;

use aequitas::systems::si::quantities::Time;
use horae::{
    events::EventSchedule,
    integration::{StepWorkspace, step_into, tableau::Euler},
    system::ExplicitSystem,
    time::{Instant, StepSize},
};
use proptest::prelude::*;

struct ConstantRate(f64);

impl ExplicitSystem<f64> for ConstantRate {
    type Error = Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        _state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        derivative.fill(self.0);
        Ok(())
    }
}

proptest! {
    #[test]
    fn euler_matches_constant_rate_closed_form(
        initial in -1.0e3_f64..1.0e3,
        rate in -1.0e2_f64..1.0e2,
        step_value in 1.0e-6_f64..1.0,
    ) {
        let start = Instant::new(Time::from_base(0.0))
            .expect("invariant: finite fixture");
        let step = StepSize::new(Time::from_base(step_value))
            .expect("invariant: generated positive finite step");
        let mut output = [0.0];
        let mut workspace = StepWorkspace::<f64, 1>::new(1)
            .expect("invariant: nonzero workspace");
        let report = step_into(
            &ConstantRate(rate),
            Euler,
            start,
            step,
            &[initial],
            &mut output,
            &mut workspace,
        )
        .expect("invariant: infallible system");

        let expected = rate.mul_add(step_value, initial);
        prop_assert_eq!(output[0].to_bits(), expected.to_bits());
        prop_assert_eq!(report.evaluations(), 1);
    }

    #[test]
    fn event_clip_endpoint_equals_crossed_event(
        start_value in -1.0e3_f64..1.0e3,
        event_offset in 1.0e-6_f64..1.0,
        overshoot in 0.0_f64..1.0,
    ) {
        let start = Instant::new(Time::from_base(start_value))
            .expect("invariant: generated finite start");
        let event = Instant::new(Time::from_base(start_value + event_offset))
            .expect("invariant: generated finite event");
        let events = [event];
        let schedule = EventSchedule::new(&events)
            .expect("invariant: singleton is sorted");
        let proposed = StepSize::new(Time::from_base(event_offset + overshoot + f64::EPSILON))
            .expect("invariant: generated positive step");
        let clipped = schedule.clip_step(start, proposed)
            .expect("invariant: generated finite endpoint");
        let endpoint = start.advance(clipped.step())
            .expect("invariant: clipped endpoint is finite");

        prop_assert_eq!(endpoint, event);
        prop_assert_eq!(clipped.event(), Some(event));
    }

}

#[test]
fn event_clip_preserves_high_magnitude_small_offset() {
    let start =
        Instant::new(Time::from_base(1.0e8_f64)).expect("invariant: finite high-magnitude start");
    let event = Instant::new(Time::from_base(1.0e8_f64 + 1.0e-6_f64))
        .expect("invariant: representable event offset");
    assert!(
        event > start,
        "invariant: high-magnitude offset is representable"
    );
    let events = [event];
    let schedule = EventSchedule::new(&events).expect("invariant: singleton is sorted");
    let proposed =
        StepSize::new(Time::from_base(2.0e-6_f64)).expect("invariant: positive proposed step");

    let clipped = schedule
        .clip_step(start, proposed)
        .expect("invariant: finite clipped endpoint");
    let endpoint = start
        .advance(clipped.step())
        .expect("invariant: finite reconstructed endpoint");

    assert_eq!(clipped.event(), Some(event));
    assert_eq!(endpoint, event);
}
