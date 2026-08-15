# Adaptive Control

Horae provides a mixed absolute-relative adaptive step-size controller for
error-based acceptance/rejection and step scaling. The controller is
domain-agnostic and allocation-free.

## The Adaptive Controller

An `AdaptiveController<T>` accepts five parameters:

```rust
# extern crate horae;
# use horae::adaptive::AdaptiveController;
# fn main() -> Result<(), Box<dyn core::error::Error>> {

let controller = AdaptiveController::<f64>::new(
    1.0e-8,  // absolute_tolerance
    1.0e-5,  // relative_tolerance
    0.9,     // safety_factor
    0.2,     // minimum_scale
    5.0,     // maximum_scale
)?;
# let _ = controller;
# Ok(())
# }
```

### Parameters

**absolute_tolerance** and **relative_tolerance**:
The normalized error is computed as:
```text
normalized_error = absolute_error / (absolute_tolerance + relative_tolerance * reference_scale)
```

This mixed scale combines:
- An absolute floor for tiny solution values
- A relative (proportional) component for large values

**safety_factor**:
A multiplicative guard (∈ (0, 1]), typically 0.9. The suggested scale is:
```text
suggested_scale = safety * normalized_error^(-1/(ORDER+1))
```

A smaller safety factor produces more conservative steps (more rejections).

**minimum_scale** and **maximum_scale**:
Bounds on the step multiplier (typically ∈ [0.1, 1] and [1, 5], respectively).
Even if error suggests a huge step, it is clamped to `maximum_scale * current_step`.

## Step Assessment

To assess a proposed step:

```rust
# extern crate aequitas;
# extern crate horae;
# use aequitas::systems::si::quantities::Time;
# use horae::{adaptive::AdaptiveController, time::StepSize};
# fn main() -> Result<(), Box<dyn core::error::Error>> {
# let controller = AdaptiveController::<f64>::new(1.0e-8, 1.0e-5, 0.9, 0.2, 5.0)?;
# let step_size = StepSize::new(Time::from_base(0.1))?;

let error = 0.005;
let scale = 1.0;  // reference state scale (e.g., max(|y|))

let assessment = controller.assess::<4>(step_size, error, scale)?;
# let _ = assessment;
# Ok(())
# }
```

The assessment returns an allocation-free report with:

- **decision**: `StepDecision::Accept` or `Reject`
- **normalized_error**: the computed error metric
- **scale**: the suggested step multiplier
- **suggested_step**: a pre-scaled new step size

A step is accepted if `normalized_error ≤ 1.0`. Otherwise, it is rejected.

## Accept/Reject Workflow

A typical adaptive loop:

```rust
# extern crate aequitas;
# extern crate horae;
# use aequitas::systems::si::quantities::Time;
# use horae::{
#     adaptive::{AdaptiveController, StepDecision},
#     integration::{step_into, tableau::Rk4, StepWorkspace},
#     system::ExplicitSystem,
#     time::{Instant, StepSize},
# };
# struct Decay;
# #[derive(Debug)]
# struct UnsupportedDecision;
# impl core::fmt::Display for UnsupportedDecision {
#     fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
#         formatter.write_str("unsupported adaptive decision")
#     }
# }
# impl core::error::Error for UnsupportedDecision {}
# impl ExplicitSystem<f64> for Decay {
#     type Error = core::convert::Infallible;
#     fn evaluate(
#         &self,
#         _time: Instant<f64>,
#         state: &[f64],
#         derivative: &mut [f64],
#     ) -> Result<(), Self::Error> {
#         for (slope, value) in derivative.iter_mut().zip(state) {
#             *slope = -*value;
#         }
#         Ok(())
#     }
# }
# fn main() -> Result<(), Box<dyn core::error::Error>> {
# let controller = AdaptiveController::<f64>::new(1.0e-8, 1.0e-5, 0.9, 0.2, 5.0)?;
# let mut current_time = Instant::new(Time::from_base(0.0))?;
# let mut step_size = StepSize::new(Time::from_base(0.01))?;
# let mut state = [1.0];
# let mut trial_state = [0.0];
# let mut workspace = StepWorkspace::<f64, 4>::new(1)?;
# let system = Decay;

loop {
    // Attempt a trial step
    let report = step_into(
        &system,
        Rk4,
        current_time,
        step_size,
        &state,
        &mut trial_state,
        &mut workspace,
    )?;
    
    // Estimate error (example: compare with lower-order method)
    let estimated_error = (state[0] - trial_state[0]).abs();
    
    // Assess
    let reference_scale = state[0].abs().max(1.0e-12);
    let assessment = controller.assess::<4>(step_size, estimated_error, reference_scale)?;
    
    match assessment.decision() {
        StepDecision::Accept => {
            current_time = report.end();
            state.copy_from_slice(&trial_state);
            step_size = assessment.suggested_step();
            break;
        }
        StepDecision::Reject => {
            // Shrink and retry
            step_size = assessment.suggested_step();
            continue;
        }
        _ => return Err(UnsupportedDecision.into()),
    }
}
# Ok(())
# }
```

## Scalar Coverage

The adaptive controller is generic over `FloatElement`, supporting both `f64` and `f32`.
Precision bounds are maintained appropriately:

- `f64`: 4–8 rounding errors per tolerance check
- `f32`: 8–12 rounding errors per tolerance check (due to lower precision)

Both monomorphizations have identical semantics and are tested to the same
behavioral contract.

## Non-Finite Handling

The controller rejects non-finite observations:

```rust
# extern crate aequitas;
# extern crate horae;
# use aequitas::systems::si::quantities::Time;
# use horae::{adaptive::AdaptiveController, time::StepSize};
# fn main() -> Result<(), Box<dyn core::error::Error>> {
# let controller = AdaptiveController::<f64>::new(1.0e-8, 1.0e-5, 0.9, 0.2, 5.0)?;
# let overflow_controller = AdaptiveController::<f64>::new(
#     1.0e-8,
#     f64::MAX,
#     0.9,
#     0.2,
#     5.0,
# )?;
# let step = StepSize::new(Time::from_base(0.1))?;

// NaN or infinite error
controller.assess::<4>(step, f64::NAN, 1.0)
    .expect_err("NaN observation");  // AdaptiveError::NonFiniteObservation

// Overflow in tolerance calculation
overflow_controller.assess::<4>(step, 1.0, 2.0)
    .expect_err("overflow");  // AdaptiveError::NonFiniteTolerance
# Ok(())
# }
```

Invalid method orders (0 or u32::MAX) are also rejected.
