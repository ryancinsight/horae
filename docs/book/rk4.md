# RK4 and tableaus

Horae represents an explicit Runge--Kutta method as a zero-sized tableau
marker. `Euler`, `Midpoint`, and `Rk4` implement the sealed
`integration::tableau::ExplicitTableau<STAGES>` contract. The stage count is a
const generic, so the method and its workspace agree at compile time.

For classical RK4 the four stages use the usual coefficients:

```text
k1 = f(t,                 y)
k2 = f(t + h/2,            y + h k1/2)
k3 = f(t + h/2,            y + h k2/2)
k4 = f(t + h,              y + h k3)
y' = y + h (k1 + 2 k2 + 2 k3 + k4) / 6
```

The public stepping function is `integration::step_into`. Allocate a
`StepWorkspace<T, 4>` once, then reuse it for repeated calls; the recurrence
uses borrowed slices and performs no per-step allocation:

The following is a focused API fragment and assumes the `Decay` system from
[Explicit systems](explicit_systems.md); the compiled end-to-end example is
[`ordered_decay.rs`](../../examples/ordered_decay.rs).

```rust,ignore
use aequitas::systems::si::quantities::Time;
use horae::{
    integration::{step_into, tableau::Rk4, StepWorkspace},
    time::{Instant, StepSize},
};

let start = Instant::new(Time::from_base(0.0))?;
let step = StepSize::new(Time::from_base(0.1))?;
let mut workspace = StepWorkspace::<f64, 4>::new(1)?;
let mut output = [0.0];
let report = step_into(
    &Decay { rate: 1.0 },
    Rk4,
    start,
    step,
    &[1.0],
    &mut output,
    &mut workspace,
)?;
assert_eq!(report.evaluations(), 4);
```

`StepReport` records the start and end instants, the accepted step, and the
number of right-hand-side evaluations. `StepError` distinguishes shape
mismatches, invalid stage/end times, and system failures. A consumer chooses
the tableau marker and retains ownership of the state transition around the
returned output.

The runnable [ordered-decay example](examples/ordered_decay.md) is the
repository's compiled demonstration of this path. New methods should extend
the provider-owned tableau set rather than duplicate the stepping recurrence in
a consumer.
