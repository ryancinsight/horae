# Adaptive control

`adaptive::AdaptiveController<T>` applies a caller-supplied error observation
to a mixed absolute/relative tolerance. It does not estimate error itself and
does not own a norm or an embedded method. The caller supplies:

- `absolute_error`, the aggregate error for the trial;
- `reference_scale`, the scale of the state used by the relative tolerance;
- the method order as a const generic on `assess`.

The controller computes

```text
normalized_error = |absolute_error|
                   / (absolute_tolerance + relative_tolerance * |reference_scale|)
```

and returns an `AdaptiveReport`. Values at or below one are accepted; values
above one are rejected. The report also contains the clamped safety-scaled
factor and a validated suggested `StepSize`:

The following is a focused API fragment; the names `step`, `reference_scale`,
`commit_trial_state`, and `retry_with` represent the surrounding consumer's
trial-state loop:

```rust,ignore
use horae::{adaptive::{AdaptiveController, StepDecision}, time::StepSize};

let controller = AdaptiveController::new(
    1.0e-8, // absolute tolerance
    1.0e-5, // relative tolerance
    0.9,    // safety factor
    0.2,    // minimum scale
    5.0,    // maximum scale
)?;
let report = controller.assess::<4>(step, 2.0e-7, reference_scale)?;
match report.decision() {
    StepDecision::Accept => commit_trial_state(),
    StepDecision::Reject => retry_with(report.suggested_step()),
}
```

A zero observed error selects the configured maximum scale. Non-finite
observations, tolerance overflow, invalid method orders, and invalid suggested
steps are rejected with `AdaptiveError`; this keeps the time contract finite
before the caller retries.

Adaptive policy is intentionally separate from stepping. A consumer may use an
embedded pair or another independently owned estimator, feed its aggregate
observation to Horae, and decide when to commit or discard its own trial state.
The full executable example demonstrates this policy after an event-clipped RK4
step.
