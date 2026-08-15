# Explicit Systems

Horae owns a reusable contract for first-order initial-value problems.
The time-stepping machinery stays domain-agnostic by accepting a
caller-supplied system through a thin seam: the `ExplicitSystem<T>` trait.

## The ExplicitSystem Trait

An explicit system is a callable that evaluates a right-hand side (RHS) vector
at a given time and state:

```rust
# extern crate horae;
# use horae::time::Instant;

pub trait ExplicitSystem<T> {
    type Error;

    fn evaluate(
        &self,
        time: Instant<T>,
        state: &[T],
        derivative: &mut [T],
    ) -> Result<(), Self::Error>;
}
```

The caller owns:
- **State**: the solution vector `y` that progresses through time
- **Derivatives**: storage for the system's evaluation `dy/dt = f(t, y)`
- **Error handling**: the system's error type, preserved through Horae's API

Horae owns:
- **Time values**: the `Instant<T>` is guaranteed finite by its constructor
- **Orchestration**: Horae decides when to evaluate the system and how many
  times per step
- **Storage**: the derivative and stage buffers are allocated and managed by
  Horae through `StepWorkspace<T, STAGES>`

This separation keeps Horae free from equation knowledge, spatial
discretization, and domain-specific error handling.

## Implementing ExplicitSystem

Consider a simple exponential decay system: `dy/dt = -λ·y`.

```rust
# extern crate horae;

use horae::system::ExplicitSystem;
use horae::time::Instant;

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
```

The system object itself can hold parameters (like `rate: f64` above) or
reference external state. Horae borrows it immutably and calls `evaluate`
multiple times per step for higher-order methods.

## Scalar Types

Horae is generic over any `FloatElement` scalar: currently `f64` and `f32`.
Both are supported with identical stability and error-handling semantics:

```rust
# extern crate horae;
# use horae::system::ExplicitSystem;
# use horae::time::Instant;
# struct Decay {
#     rate: f32,
# }

impl ExplicitSystem<f32> for Decay {
    type Error = core::convert::Infallible;

    fn evaluate(
        &self,
        _time: Instant<f32>,
        state: &[f32],
        derivative: &mut [f32],
    ) -> Result<(), Self::Error> {
        for (slope, value) in derivative.iter_mut().zip(state) {
        *slope = -self.rate * value;
        }
        Ok(())
    }
}
```

## Error Handling

The system's error type is preserved through the step. If integration fails
to produce a valid result, any system error is propagated back to the caller:

```rust
# extern crate horae;
# use horae::system::ExplicitSystem;
# use horae::time::Instant;
# struct MySystem;

#[derive(Debug)]
pub enum SystemError {
    InvalidParameter,
    ComputationFailed,
}

impl ExplicitSystem<f64> for MySystem {
    type Error = SystemError;

    fn evaluate(
        &self,
        time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        if state.is_empty() {
            return Err(SystemError::InvalidParameter);
        }
        // ... computation ...
        Ok(())
    }
}
```

When `step_into` calls your system and receives an error, that error is
returned immediately without consuming workspace allocations.
