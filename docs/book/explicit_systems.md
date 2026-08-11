# Explicit systems

Horae advances a caller-owned first-order system. The provider does not own an
ODE, a mesh, or a state container; it asks an implementation of
`system::ExplicitSystem<T>` to evaluate `dy/dt` into a caller-provided slice.

The contract is deliberately small:

- `Instant<T>` carries the finite simulation time;
- `state` is an immutable borrowed view of the trial state;
- `derivative` is the mutable output buffer and must have the same dimension;
- the associated `Error` lets a domain reject an evaluation without coupling
  Horae to that domain's error vocabulary.

A domain implementation can therefore retain its own equations and storage.
The following is a focused, non-standalone API fragment:

```rust,ignore
use horae::{system::ExplicitSystem, time::Instant};

struct Decay { rate: f64 }

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

The implementation should enforce any domain-specific length checks; the
trait itself cannot encode that relationship. `step_into` performs the
structural checks it can see before the first evaluation and reports
`StepError::DimensionMismatch` for an output or workspace mismatch. A system
may still return its own error for semantic validation or a domain failure.

This boundary keeps integration policy reusable. Athena, Harmonia, and other
consumers can implement the trait around their existing equations without
moving CFL criteria, coupling, arrays, or device execution into Horae.

For a complete executable implementation, see
[`examples/ordered_decay.rs`](../../examples/ordered_decay.rs), which uses
the same contract with RK4, event clipping, adaptive control, and subcycling.
