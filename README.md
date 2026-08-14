# Horae

[![CI](https://github.com/ryancinsight/horae/actions/workflows/ci.yml/badge.svg)](https://github.com/ryancinsight/horae/actions/workflows/ci.yml)

Horae is the Atlas time-integration policy and orchestration foundation. It
owns typed simulation time, explicit Runge--Kutta stepping, adaptive
accept/reject policy, event-boundary clipping, and compile-time subcycle ratios.
The name refers to the Horae, the Greek personifications of ordered time.

Horae is independently versioned and consumed by the
[Atlas multiphysics stack](https://github.com/ryancinsight/atlas). Its
cross-repository ownership and migration boundary is recorded in
[Atlas ADR 0022](https://github.com/ryancinsight/atlas/blob/main/docs/adr/0022-horae-athena-provider-extraction.md).

## Boundary

Horae owns:

- finite `Instant<T>` and positive `StepSize<T>` values over
  `aequitas::Time<T>`;
- a borrowed-slice `ExplicitSystem<T>` seam with an associated error;
- const-generic explicit tableaus and one backend-neutral stepping recurrence;
- reusable stage workspaces;
- error-based adaptive decisions and reports;
- borrowed, sorted event schedules with authoritative event-boundary clipping;
- zero-sized, const-generic subcycle-ratio plans.

Horae does not own equations, spatial discretization, stability or CFL
criteria, nonlinear or implicit solvers, coupling invocation, arrays, task
scheduling, or device execution. Consumers supply equations through
`ExplicitSystem`; Aequitas owns units, Eunomia owns scalar law, Leto owns CPU
arrays, and Hephaestus owns GPU execution.

Downstream domain and coupling packages implement `ExplicitSystem` and retain
their equations, stability constraints, and state storage. They may consume
Horae's typed time, event, and subcycle contracts without introducing a reverse
dependency from Horae to a domain, scheduler, array, or device provider.

## Example

```rust
use aequitas::systems::si::quantities::Time;
use horae::{
    integration::{StepWorkspace, step_into, tableau::Rk4},
    system::ExplicitSystem,
    time::{Instant, StepSize},
};

struct Decay;

impl ExplicitSystem<f64> for Decay {
    type Error = core::convert::Infallible;

    fn evaluate(
        &self,
        _time: Instant<f64>,
        state: &[f64],
        derivative: &mut [f64],
    ) -> Result<(), Self::Error> {
        for (rate, value) in derivative.iter_mut().zip(state) {
            *rate = -*value;
        }
        Ok(())
    }
}

# // dyn exception: top-level example error aggregation is outside integration paths.
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let start = Instant::new(Time::from_base(0.0))?;
let step = StepSize::new(Time::from_base(0.1))?;
let mut workspace = StepWorkspace::<f64, 4>::new(1)?;
let mut next = [0.0];
let report = step_into(
    &Decay,
    Rk4,
    start,
    step,
    &[1.0],
    &mut next,
    &mut workspace,
)?;

assert_eq!(report.evaluations(), 4);
// RK4 matches exp(-h) through h⁴. For h=0.1 the alternating-series
// remainder is bounded by h⁵/5!; the epsilon term covers evaluated rounding.
let truncation_bound = 0.1_f64.powi(5) / 120.0;
let rounding_bound = 32.0 * f64::EPSILON;
assert!(
    (next[0] - (-0.1_f64).exp()).abs()
        <= truncation_bound + rounding_bound
);
# Ok(())
# }
```

The runnable [`ordered_decay`](examples/ordered_decay.rs) program additionally
demonstrates event clipping, adaptive policy, and a subcycle ratio.

## Architecture

```text
src/
├── adaptive/
│   ├── controller.rs   # mixed absolute-relative scale policy
│   ├── decision.rs     # accept/reject domain result
│   └── report.rs       # allocation-free observation
├── events/
│   ├── schedule.rs     # validated borrowed ordering and binary search
│   └── report.rs       # exact boundary hit
├── integration/
│   ├── tableau/
│   │   ├── methods.rs  # Euler, midpoint, and RK4 ZSTs
│   │   └── model.rs    # const-generic coefficient contract
│   ├── stepper.rs      # single monomorphized explicit recurrence
│   └── workspace.rs    # caller-owned contiguous stage storage
├── subcycling/
│   └── plan.rs         # const-generic ratio ZST only
├── system/
│   └── explicit.rs     # borrowed slice system seam
└── time/
    ├── instant.rs      # finite Aequitas time
    └── step_size.rs    # positive Aequitas duration
```

`lib.rs` and every `mod.rs` are manifests. Tableaus and subcycle plans are
zero-sized. `StepWorkspace<T, STAGES>` allocates at construction and repeated
`step_into` calls reuse it without allocation. Scalar arithmetic executes in
the selected `T: eunomia::FloatElement`; coefficient metadata is converted at
the kernel boundary.

The crate is `no_std + alloc` in both feature configurations. Its default
`std` feature only propagates standard-library support to Aequitas and
Eunomia; `--no-default-features` keeps the same numerical and orchestration
contracts without a standard-library dependency.

The explicit recurrence follows the Butcher tableau formulation in
Hairer, Nørsett, and Wanner,
[*Solving Ordinary Differential Equations I*, Chapter II](https://doi.org/10.1007/978-3-662-12607-3).
The package intentionally supplies only the three current method markers; new
methods extend the sealed provider-owned coefficient set rather than cloning
the recurrence.

## Verification

The committed gates are:

```sh
cargo fmt --check
cargo check --no-default-features
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features
cargo test --doc --all-features
cargo doc --no-deps --all-features
cargo run --example ordered_decay
cargo deny check
```

Behavioral tests compare all three method instantiations with independent
closed-form or polynomial oracles. Property tests cover constant-rate
integration and event clipping. Layout tests prove policy markers are
zero-sized and time newtypes remain transparent. A dedicated instrumented
allocator test proves repeated `step_into` calls allocate, reallocate, and
deallocate zero times after workspace construction.

## License

Licensed under either the MIT License or Apache License 2.0.
