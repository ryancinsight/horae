# Subcycle plans

`subcycling::SubcyclePlan<const RATIO: usize>` is a zero-sized policy marker.
It validates a positive compile-time ratio and derives one fine `StepSize` from
a parent step. For example, a ratio of four creates four equal fine steps. The following is
a focused, non-standalone API fragment:

```rust,ignore
use aequitas::systems::si::quantities::Time;
use horae::{subcycling::SubcyclePlan, time::StepSize};

let parent = StepSize::new(Time::from_base(0.4))?;
let plan = SubcyclePlan::<4>::new()?;
let child = plan.child_step(parent)?;
assert!((child.as_time().as_base() - 0.1).abs() < f64::EPSILON);
assert_eq!(plan.ratio(), 4);
```

`SubcycleError::ZeroRatio` rejects `SubcyclePlan<0>`, and ratios above
`u32::MAX` are rejected before coefficient conversion. Reduced-precision
underflow or another invalid result from division is reported as
`SubcycleError::ChildStep`.

The plan expresses only the ratio. It does not call a coarse or fine system,
advance state, synchronize buffers, or decide how coupling is applied. Athena,
Harmonia, and other consumers retain those responsibilities and can combine a
plan with their own scheduler, state storage, and convergence policy. This
keeps the compile-time ratio reusable without introducing a reverse dependency
from Horae to a domain or execution provider.
