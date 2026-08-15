# RK4 and Tableaux

Horae implements explicit one-step methods using Butcher tableaus, a compact
notation for Runge–Kutta methods. The mathematics follows Hairer, Nørsett,
and Wanner's *Solving Ordinary Differential Equations I*, Chapter II.

## The Butcher Tableau

A Butcher tableau encodes a k-stage method using the layout:

```text
  c1 | a11  a12  ...  a1k
  c2 | a21  a22  ...  a2k
  .. | ...
  ck | ak1  ak2  ...  akk
  ---+------------------
     | b1   b2   ...  bk
```

where:
- `c_i` are the stage time fractions (relative to the step size)
- `a_ij` are the coupling coefficients
- `b_i` are the step weights

The computation proceeds as:

```text
y₁ = f(t + c₁·h, y + h·a₁₁·y₁')
y₂ = f(t + c₂·h, y + h·(a₂₁·y₁' + a₂₂·y₂'))
...
y_{n+1} = y_n + h·(b₁·y₁' + b₂·y₂' + ... + bₖ·yₖ')
```

where `y_i'` denotes the i-th stage derivative.

## Supported Methods

Horae provides three sealed zero-sized method markers:

### Euler (1st order, 1 stage)

The simplest explicit method:

```text
  0 | 
  ---+--
    | 1
```

Implementation:

```rust
# extern crate aequitas;
# extern crate horae;
# use aequitas::systems::si::quantities::Time;
# use horae::{
#     integration::{step_into, StepWorkspace},
#     system::ExplicitSystem,
#     time::{Instant, StepSize},
# };
# struct ConstantSystem;
# impl ExplicitSystem<f64> for ConstantSystem {
#     type Error = core::convert::Infallible;
#     fn evaluate(
#         &self,
#         _time: Instant<f64>,
#         _state: &[f64],
#         derivative: &mut [f64],
#     ) -> Result<(), Self::Error> {
#         derivative.fill(0.0);
#         Ok(())
#     }
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let system = ConstantSystem;
# let start = Instant::new(Time::from_base(0.0))?;
# let step_size = StepSize::new(Time::from_base(0.1))?;
# let state = [1.0];
# let mut next_state = [0.0];
# let mut workspace = StepWorkspace::<f64, 1>::new(1)?;

use horae::integration::tableau::Euler;

let report = step_into(
    &system,
    Euler,
    start,
    step_size,
    &state,
    &mut next_state,
    &mut workspace,
)?;
# let _ = report;
# Ok(())
# }
```

**Cost**: 1 system evaluation per step
**Accuracy**: O(h) for smooth problems
**Stability**: limited; suitable for rough, nonlinear dynamics only

### Explicit Midpoint (2nd order, 2 stages)

```text
  0   |  
  1/2 | 1/2
  ----+----------
      | 0   1
```

Implementation:

```rust
# extern crate aequitas;
# extern crate horae;
# use aequitas::systems::si::quantities::Time;
# use horae::{
#     integration::{step_into, StepWorkspace},
#     system::ExplicitSystem,
#     time::{Instant, StepSize},
# };
# struct ConstantSystem;
# impl ExplicitSystem<f64> for ConstantSystem {
#     type Error = core::convert::Infallible;
#     fn evaluate(
#         &self,
#         _time: Instant<f64>,
#         _state: &[f64],
#         derivative: &mut [f64],
#     ) -> Result<(), Self::Error> {
#         derivative.fill(0.0);
#         Ok(())
#     }
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let system = ConstantSystem;
# let start = Instant::new(Time::from_base(0.0))?;
# let step_size = StepSize::new(Time::from_base(0.1))?;
# let state = [1.0];
# let mut next_state = [0.0];
# let mut workspace = StepWorkspace::<f64, 2>::new(1)?;

use horae::integration::tableau::Midpoint;

let report = step_into(
    &system,
    Midpoint,
    start,
    step_size,
    &state,
    &mut next_state,
    &mut workspace,
)?;
# let _ = report;
# Ok(())
# }
```

**Cost**: 2 system evaluations per step
**Accuracy**: O(h²)
**Stability**: moderately larger than Euler

### Classical RK4 (4th order, 4 stages)

The most widely used explicit method:

```text
  0   |  
  1/2 | 1/2
  1/2 | 0   1/2
  1   | 0   0   1
  ----+------------------------
      | 1/6 1/3 1/3 1/6
```

Implementation:

```rust
# extern crate aequitas;
# extern crate horae;
# use aequitas::systems::si::quantities::Time;
# use horae::{
#     integration::{step_into, StepWorkspace},
#     system::ExplicitSystem,
#     time::{Instant, StepSize},
# };
# struct ConstantSystem;
# impl ExplicitSystem<f64> for ConstantSystem {
#     type Error = core::convert::Infallible;
#     fn evaluate(
#         &self,
#         _time: Instant<f64>,
#         _state: &[f64],
#         derivative: &mut [f64],
#     ) -> Result<(), Self::Error> {
#         derivative.fill(0.0);
#         Ok(())
#     }
# }
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let system = ConstantSystem;
# let start = Instant::new(Time::from_base(0.0))?;
# let step_size = StepSize::new(Time::from_base(0.1))?;
# let state = [1.0];
# let mut next_state = [0.0];
# let mut workspace = StepWorkspace::<f64, 4>::new(1)?;

use horae::integration::tableau::Rk4;

let report = step_into(
    &system,
    Rk4,
    start,
    step_size,
    &state,
    &mut next_state,
    &mut workspace,
)?;
# let _ = report;
# Ok(())
# }
```

**Cost**: 4 system evaluations per step
**Accuracy**: O(h⁴) for smooth problems
**Stability**: approximately 2.8 times CFL compared to forward Euler

## Method Selection

Choose based on smoothness and desired accuracy:

- **Euler**: Rough, highly nonlinear, or stiff dynamics (with stability
  restrictions)
- **Midpoint**: Smooth, moderately stiff problems; balance between cost and
  accuracy
- **RK4**: Smooth problems where O(h⁴) accuracy justifies 4 system evaluations

The `step_into` function is monomorphic: the method is known at compile time,
and the recurrence specializes for each method's stage count and coefficients.

## Coefficients are Sealed

New methods cannot be defined by implementing a trait. The coefficient set is
closed and provider-owned. This ensures correctness: wrong coefficients remain
an external mistake, not a silently miscompiled integration. To add a new
method, file a feature request upstream.
