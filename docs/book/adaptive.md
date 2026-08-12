# Adaptive Control

Horae provides a mixed absolute-relative adaptive step-size controller for
error-based acceptance/rejection and step scaling. The controller is
domain-agnostic and allocation-free.

## The Adaptive Controller

An `AdaptiveController<T>` accepts five parameters:

```rust
use horae::adaptive::AdaptiveController;

let controller = AdaptiveController::<f64>::new(
    1.0e-8,  // absolute_tolerance
    1.0e-5,  // relative_tolerance
    0.9,     // safety_factor
    0.2,     // minimum_scale
    5.0,     // maximum_scale
)?;
```

### Parameters

**absolute_tolerance** and **relative_tolerance**:
The normalized error is computed as:
```
normalized_error = absolute_error / (absolute_tolerance + relative_tolerance * reference_scale)
```

This mixed scale combines:
- An absolute floor for tiny solution values
- A relative (proportional) component for large values

**safety_factor**:
A multiplicative guard (∈ (0, 1]), typically 0.9. The suggested scale is:
```
suggested_scale = safety * normalized_error^(-1/(ORDER+1))
```

A smaller safety factor produces more conservative steps (more rejections).

**minimum_scale** and **maximum_scale**:
Bounds on the step multiplier (typically ∈ [0.1, 1] and [1, 5], respectively).
Even if error suggests a huge step, it is clamped to `maximum_scale * current_step`.

## Step Assessment

To assess a proposed step:

```rust
let error = 0.005;
let scale = 1.0;  // reference state scale (e.g., max(|y|))

let assessment = controller.assess::<4>(step_size, error, scale)?;
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
    let estimated_error = estimate_error(&state, &trial_state);
    
    // Assess
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
    }
}
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
// NaN or infinite error
controller.assess::<4>(step, f64::NAN, 1.0)
    .expect_err("NaN observation");  // AdaptiveError::NonFiniteObservation

// Overflow in tolerance calculation
controller.assess::<4>(step, 1.0, f64::MAX)
    .expect_err("overflow");  // AdaptiveError::NonFiniteTolerance
```

Invalid method orders (0 or u32::MAX) are also rejected.
