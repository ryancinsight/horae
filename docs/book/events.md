# Event schedules

`events::EventSchedule<'a, T>` validates a borrowed, nondecreasing slice of
finite `Instant<T>` values. It stores no event list of its own, so a simulation
can keep schedule ownership in its domain or orchestration layer. The snippets
in this chapter are focused API fragments; the compiled end-to-end example is
[`ordered_decay.rs`](../../examples/ordered_decay.rs).

```rust,ignore
use aequitas::systems::si::quantities::Time;
use horae::{events::EventSchedule, time::{Instant, StepSize}};

let event = Instant::new(Time::from_base(0.25))?;
let events = [event];
let schedule = EventSchedule::new(&events)?;
let start = Instant::new(Time::from_base(0.0))?;
let proposed = StepSize::new(Time::from_base(0.3))?;
let clip = schedule.clip_step(start, proposed)?;
assert!(clip.was_shortened());
assert_eq!(clip.event(), Some(event));
assert!((clip.step().as_time().as_base() - 0.25).abs() < f64::EPSILON);
```

`next_after(start)` uses binary search and skips events equal to `start`, so
duplicate notifications at the current boundary cannot produce a zero-length
step. `clip_step` computes the proposed endpoint, returns the original step
when no event is crossed, and otherwise returns a positive step ending exactly
at the next event. `EventClip::event` identifies the boundary and
`was_shortened` distinguishes a shortened proposal from an unchanged one.

If a schedule is not nondecreasing, construction fails with
`ScheduleError::Unsorted` and reports the first violating index. Overflow while
forming a proposed endpoint is reported as `TimeError::NonFinite`.

Event notification and post-event state updates remain consumer policy. Horae
only supplies the validated ordering and exact time boundary; it does not
invoke callbacks or own the event payloads.
