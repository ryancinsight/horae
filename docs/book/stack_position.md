# Position in the Atlas Stack

Horae is the **time-integration policy and orchestration foundation** for the
Atlas multiphysics stack. It owns the temporal contracts, not the equations or
execution.

## Boundary: What Horae Owns

Horae is responsible for:

- **Time values**: `Instant<T>` (finite time points) and `StepSize<T>` (finite,
  strictly positive durations) over Aequitas physical units.
- **Explicit integration**: Runge–Kutta tableaus (Euler, midpoint, RK4, and
  Dormand–Prince 5(4)) with staged evaluation and shared embedded stages.
- **Stage storage**: A reusable, caller-owned `StepWorkspace<T, STAGES>` that
  allocates once and steps many times without further allocation.
- **Error assessment**: Mixed absolute-relative adaptive control and
  accept/reject policy.
- **Event clipping**: No-overstep detection for scheduled instants; the event
  value is returned as the authoritative endpoint (no interpolation).
- **Subcycling**: Fixed-ratio nested stepping at integer multiples.
- **Error propagation**: Every operation preserves the system's error type.

## Boundary: What Horae Does Not Own

Consumers own:

- **Equations**: The `ExplicitSystem<T>` seam lets each domain implement its
  own right-hand side. Horae never contains physics, chemical kinetics, or
  fluid dynamics.
- **Stability criteria**: CFL conditions, stiffness limits, and implicit
  solvers are domain-specific.
- **Spatial discretization**: Mesh, finite-element bases, or spectral methods
  are orthogonal to time stepping.
- **Arrays and storage**: Leto owns CPU arrays and memory layout. Hephaestus
  owns GPU device allocation.
- **Task scheduling**: Moirai owns runtime parallelism and execution.

## Dependency Structure

```text
┌──────────────────────────────────────────┐
│     Consumer domain code (when present)  │
│        implements ExplicitSystem<T>      │
│        calls step_into(system, ...)      │
└─────────────────────┬────────────────────┘
                      │
        ┌─────────────┴──────────────┐
        │                            │
    ┌───▼─────┐            ┌────────▼──┐
    │  Horae  │            │  Leto      │
    │(time)   │            │(arrays)    │
    └───┬─────┘            └────────┬───┘
        │                           │
        └──────────────┬────────────┘
                       │
              ┌────────▼──────┐
              │   Aequitas    │
              │  (units, time)│
              └───────────────┘
              
        ┌──────────────────┐
        │  Hephaestus      │
        │  (GPU execution) │
        └──────────────────┘
```

- **Horae → Aequitas**: Time types and units
- **Horae → Eunomia**: Scalar operations (multiplication, exponentiation)
- **Domain → Horae**: Equations through `ExplicitSystem<T>`
- **Domain → Leto**: Array allocation and access patterns
- **Domain → Hephaestus**: GPU kernels (orthogonal to Horae)

Horae does **not** depend on Leto or Hephaestus. Its slice-based contracts
permit current CPU arrays and future accelerator backends without code
changes.

## Current consumer status

Harmonia is the live coupling consumer of Horae's typed time and subcycling
contracts, and Helios uses the validated `StepSize` boundary for delivery
kinematics. The `ExplicitSystem<T>` and stepping APIs are currently exercised
by Horae's provider tests and example; no CFDrs or Kwavers production call
site is yet migrated to them. That migration is consumer-owned. An implicit
or nonlinear integration policy remains gated on a second concrete
residual/Jacobian consumer, as recorded in ADR 0001.

## Integration with Other Atlas Components

### With Athena (Convergence Policy)

Athena owns the outer nonlinear iteration loop. Horae provides time-stepped
solutions. Athena decides when to converge by (typically) tracking residual
norms across iterations; Horae provides fast, accurate time steps.

### With Themis (Topology)

Themis owns spatial topology and mesh operations. Horae accepts no topological
knowledge—it only cares that consumers implement `ExplicitSystem<T>`.

### With Consus (I/O and Serialization)

Consus owns checkpointing and output. Horae provides `Instant<T>` values that
Consus can label checkpoints with precise time stamps.

### With Tyche (Uncertainty Quantification)

Tyche may wrap systems to track uncertainty through coupled or ensemble
simulations. Each system variant implements `ExplicitSystem<T>` and calls Horae
normally.

## API Stability

Horae is `publish = false` in its workspace but exported as a Git dependency.
Its public API is stable; breaking changes require explicit coordination with
consuming repositories.

The trait contracts (`ExplicitSystem<T>`, the tableau model, workspace layout)
are permanent. Minor additions (new methods, new scalar support) are
non-breaking.

## No_std and Portable

Horae builds as `no_std + alloc` with `--no-default-features`. Only
`StepWorkspace` construction allocates; all stepping and policy decisions are
allocation-free. This suits embedded and real-time environments.

The crate has no platform-specific code. Correctness relies only on Rust's
type system and explicit invariants (finite times, positive steps).
