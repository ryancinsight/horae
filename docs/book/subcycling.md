# Subcycle Plans

Horae provides a const-generic contract for fixed-ratio subcycling: nested
stepping at integer multiples of the parent step. Subcycle plans are
zero-sized policy markers; they derive fine-step durations but invoke no
coupled systems themselves.

## Fixed-Ratio Subcycling

A `SubcyclePlan<const RATIO: usize>` encodes a fixed subcycle ratio:

```rust
use horae::subcycling::SubcyclePlan;

// Subdivide each parent step into 4 child steps
let plan = SubcyclePlan::<4>::new()?;
```

The ratio is const-generic so it is known at compile time. Once constructed, the
plan is zero-sized—no storage overhead.

## Deriving the Child Step

Given a parent step, the plan derives the child step:

```rust
use horae::time::StepSize;
use aequitas::systems::si::quantities::Time;

let parent = StepSize::new(Time::from_base(1.0))?;
let child = plan.child_step(parent)?;

// For RATIO=4, child is 0.25
assert_eq!(plan.ratio(), 4);
```

The child step is guaranteed:
- To be finite and positive (if parent is)
- To use the representable reciprocal of `RATIO` in the chosen scalar type
- To carry a bounded rounding error when repeated `RATIO` times

The repeated child duration is not generally bit-identical to the parent.
For positive `f64` values with `RATIO = 3`, the reciprocal, scaling
multiplication, and two sequential additions give the derived bound
`|reconstructed - parent| <= gamma_4 * |parent|`, where
`gamma_n = n * u / (1 - n * u)` and `u = f64::EPSILON / 2`. This assumes
round-to-nearest with no overflow or underflow. A coupling boundary should
use its typed instant or event value rather than relying on bit identity of a
reconstructed duration.

For other ratios and scalar types, the same rule applies: reconstructing a
parent from repeated rounded child durations is approximate, and the bound
must use that scalar's machine epsilon and operation count.

Reconstructing the parent from a non-binary ratio is subject to the selected
scalar's rounding. The test suite exercises `RATIO = 3` for `f32` and `f64`
against a bound of four first-order machine-epsilon units at the parent scale.

## Validation

The plan rejects invalid ratios:

```rust
// Zero ratio is invalid
let invalid = SubcyclePlan::<0>::new();
assert_eq!(invalid, Err(SubcycleError::ZeroRatio));
```

## Typical Workflow

A subcycling loop typically nests two integrators:

```rust
// Parent step
let parent_report = step_into(
    &system,
    Rk4,
    parent_time,
    parent_step,
    &parent_state,
    &mut parent_next,
    &mut parent_workspace,
)?;

// Subcycle with RATIO=4
let plan = SubcyclePlan::<4>::new()?;
let child_step = plan.child_step(parent_step)?;

let mut child_state = parent_state.to_vec();
for _ in 0..plan.ratio() {
    let child_report = step_into(
        &system,
        Rk4,
        child_time,
        child_step,
        &child_state,
        &mut child_next,
        &mut child_workspace,
    )?;
    child_time = child_report.end();
    child_state.copy_from_slice(&child_next);
}

// Compare results for subcycling error estimation
let subcycle_error = estimate_error(&parent_next, &child_state);
```

## Accuracy Implications

Subcycling is most useful when:
- The solution requires fine resolution in a subset of the domain
- Local error scaling justifies the extra evaluations
- The parent and child methods are the same order (to avoid degraded accuracy)

A coarse parent step subdivided into `RATIO` fine steps (same method, same
order) achieves higher accuracy than a single coarse step. The additional cost
is `RATIO` system evaluations instead of one.

## Use Cases

**Multi-rate coupling**: Different subsystems evolve at different rates. A
stiff system might take 1 large step while a fast system takes 4 small steps
within the same parent interval.

**Adaptive refinement**: An initial coarse step can be repeated at finer
resolution as a fallback or to build an error estimator.

**Convergence monitoring**: Compare coarse vs. fine results to monitor local
truncation error without external error estimation.
