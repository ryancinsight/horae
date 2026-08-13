# Event Schedules

Horae provides exact event detection: when a proposed step would cross a
scheduled event, the step is clipped exactly to the event's time. No
overstepping, no interpolation artifacts.

## Sorted Event Sequences

An `EventSchedule<'a, T>` borrows a sorted slice of instants:

```rust
use horae::events::EventSchedule;
use horae::time::Instant;
use aequitas::systems::si::quantities::Time;

let events = [
    Instant::new(Time::from_base(1.0))?,
    Instant::new(Time::from_base(2.5))?,
    Instant::new(Time::from_base(3.0))?,
];

let schedule = EventSchedule::new(&events)?;
```

The schedule validates that instants are:
- Finite
- In non-decreasing order

If the slice is not sorted, `EventSchedule::new` returns a `ScheduleError::Unsorted`.

## Finding the Next Event

```rust
let next = schedule.next_after(current_time);
// Returns Option<Instant<T>>
```

`next_after` skips events equal to the current instant. This prevents
duplicate notifications: if an event occurs exactly at `current_time`, stepping
away from it and returning will not re-trigger the event.

## Clipping to Events

When proposing a step, check whether it crosses an event:

```rust
let current = Instant::new(Time::from_base(1.0))?;
let proposed = StepSize::new(Time::from_base(0.3))?;  // Would overstep to 1.3

let clipped = schedule.clip_step(current, proposed)?;
```

The clipped result contains:

- **step**: the actual step taken (shortened if it hits an event)
- **event**: `Some(instant)` if an event lies strictly inside `[current, current + step]`
- **was_shortened**: a flag indicating whether the step was clipped

### Semantics

- **No clip without crossing**: If no event lies *strictly* between `current`
  and `current + proposed`, the step is returned unchanged with `event = None`.
- **Exact endpoint hit**: If an event lies *exactly* at `current + proposed`,
  no shortening occurs (the step already reaches it exactly).
- **Duplicate skip**: Events equal to `current` are skipped and never returned
  by `clip_step`. This prevents a step of zero length after crossing an event.

`EventClip::event()` is the authoritative scheduled endpoint. `EventClip::step()`
stores the scalar difference from the current instant; floating-point consumers
must not assume that adding the difference back is a general bit-exact identity.
For positive, same-sign endpoints within a factor of two, Sterbenz's lemma
provides the exact-subtraction precondition; use the returned event value when
endpoint identity matters.

### Example

```rust
let start = Instant::new(Time::from_base(1.0))?;
let events = [
    Instant::new(Time::from_base(1.25))?,
    Instant::new(Time::from_base(2.0))?,
];
let schedule = EventSchedule::new(&events)?;

// Proposed step would reach 1.5; clipped to hit event at 1.25
let proposed = StepSize::new(Time::from_base(0.5))?;
let clipped = schedule.clip_step(start, proposed)?;
assert!(clipped.was_shortened());
assert_eq!(clipped.event(), Some(events[0]));

// Now start at 1.25; next event is at 2.0
let proposed2 = StepSize::new(Time::from_base(0.75))?;
let clipped2 = schedule.clip_step(events[0], proposed2)?;
assert!(clipped2.was_shortened());
assert_eq!(clipped2.event(), Some(events[1]));

// Exact hit: proposed step ends exactly at 2.0
let proposed3 = StepSize::new(Time::from_base(0.75))?;
let clipped3 = schedule.clip_step(events[0], proposed3)?;
assert!(!clipped3.was_shortened());  // No shortening needed
assert_eq!(clipped3.event(), Some(events[1]));
```

## Empty Schedules

An empty schedule is valid:

```rust
let empty = EventSchedule::new(&[])?;
let clipped = empty.clip_step(current, proposed)?;
// No events, so step is unchanged
assert!(!clipped.was_shortened());
assert_eq!(clipped.event(), None);
```

## Integration with Stepping

Typically, clip the step before evaluating the system:

```rust
let clipped = schedule.clip_step(current_time, adaptive_step)?;

let report = step_into(
    &system,
    Rk4,
    current_time,
    clipped.step(),  // Use clipped step
    &state,
    &mut next_state,
    &mut workspace,
)?;

if let Some(event) = clipped.event() {
    // Handle event occurrence
    println!("Event at {:?}", event);
}
```

This ensures that event times are reached exactly, enabling precise
trigger-based logic without root-finding or dense output.
