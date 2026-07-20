//! Run an RK4 step clipped to an event and inspect follow-on policies.

use aequitas::systems::si::quantities::Time;
use horae::{
    adaptive::AdaptiveController,
    events::EventSchedule,
    integration::{StepWorkspace, step_into, tableau::Rk4},
    subcycling::SubcyclePlan,
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

struct Decay {
    rate: f64,
}

impl ExplicitSystem<f64> for Decay {
    type Error = core::convert::Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        for (slope, value) in derivative.iter_mut().zip(state) {
            *slope = -self.rate * value;
        }
        Ok(())
    }
}

// dyn exception: top-level example error aggregation is outside integration paths.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::new(Time::from_base(0.0))?;
    let proposed = StepSize::new(Time::from_base(0.3))?;
    let event = Instant::new(Time::from_base(0.25))?;
    let events = [event];
    let schedule = EventSchedule::new(&events)?;
    let clipped = schedule.clip_step(start, proposed)?;

    let mut workspace = StepWorkspace::<f64, 4>::new(1)?;
    let mut next = [0.0];
    let report = step_into(
        &Decay { rate: 1.0 },
        Rk4,
        start,
        clipped.step(),
        &[1.0],
        &mut next,
        &mut workspace,
    )?;

    let controller = AdaptiveController::new(1.0e-8, 1.0e-5, 0.9, 0.2, 5.0)?;
    let adaptive = controller.assess::<4>(clipped.step(), 2.0e-7, next[0])?;
    let subcycles = SubcyclePlan::<4>::new()?;
    let fine_step = subcycles.child_step(clipped.step())?;

    println!(
        "t={:.2}, state={:.8}, evaluations={}, decision={:?}, next_dt={:.6}, fine_dt={:.6}",
        report.end().as_time().as_base(),
        next[0],
        report.evaluations(),
        adaptive.decision(),
        adaptive.suggested_step().as_time().as_base(),
        fine_step.as_time().as_base(),
    );
    Ok(())
}
