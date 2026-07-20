# ADR 0001: Own time-integration policy in Horae

- Status: Accepted
- Date: 2026-07-19
- Class: [arch] [minor]

## Context

Atlas simulation packages need a shared time boundary without moving their
equations, arrays, or execution backends into a scheduler. The reusable law is
smaller than a complete solver framework:

- time values require physical units and ordering invariants;
- explicit one-step methods share the Butcher-tableau recurrence;
- stage storage must be reusable;
- event endpoints must be reached exactly rather than crossed;
- adaptive acceptance and subcycle ratios are policy, not equation ownership.

The Runge--Kutta coefficient model and order terminology follow Hairer,
Nørsett, and Wanner,
[*Solving Ordinary Differential Equations I*, Chapter II](https://doi.org/10.1007/978-3-662-12607-3).
Butcher's coefficient formulation is the original algebraic basis:
J. C. Butcher, “Coefficients for the study of Runge-Kutta integration
processes,” *Journal of the Australian Mathematical Society* 3(2), 1963,
[doi:10.1017/S1446788700027932](https://doi.org/10.1017/S1446788700027932).

## Decision

Create Horae as one independent `no_std + alloc` crate.

- `Instant<T>` and `StepSize<T>` are transparent newtypes over
  `aequitas::systems::si::quantities::Time<T>`. Instants are finite and steps
  are finite and strictly positive.
- `ExplicitSystem<T>` accepts borrowed state and output slices and preserves
  the implementation's associated error.
- `ExplicitTableau<const STAGES: usize>` owns coefficient arrays. Sealed
  zero-sized markers provide Euler, explicit midpoint, and classical RK4.
- `step_into` is one generic recurrence over the system and tableau. Static
  dispatch occurs once at the operation boundary.
- `StepWorkspace<T, STAGES>` owns one contiguous derivative buffer and one
  stage-state buffer. No stage allocates.
- `AdaptiveController<T>` consumes caller-computed aggregate error and state
  scale. It returns an allocation-free accept/reject report and never computes
  an equation-specific estimator.
- `EventSchedule<'a, T>` validates and borrows sorted instants. Binary search
  finds the next event, and clipping uses the exact event difference.
- `SubcyclePlan<const RATIO: usize>` is a zero-sized validated ratio. It
  derives a fine step but invokes no coupled systems.
- Aequitas and Eunomia remain direct dependencies so unit and scalar ownership
  are visible. Horae defines neither vocabulary locally.

## Rejected alternatives

### Store coefficients or methods behind trait objects

Rejected because the current method set is closed and small. Const-generic
arrays preserve stage shape, markers cost no storage, and the recurrence
monomorphizes without vtables.

### Let each consumer own its step loop

Rejected because stage construction, event clipping, error propagation, and
workspace reuse would recur across simulation domains and drift.

### Move equations or CFL policy into Horae

Rejected because stability bounds and right-hand sides belong to spatial and
physical domains. Horae accepts already-validated step proposals and borrowed
system evaluation only.

### Bind workspace storage to Leto or Hephaestus

Rejected for this first CPU-neutral slice. A slice contract permits current
caller storage without leaking an array or device type into the integration
law. A future accelerator seam must preserve the same policy ownership rather
than duplicate the algorithms.

## Consequences

- The first slice covers explicit first-order systems only.
- Adaptive embedded-pair estimation, dense output, implicit integration, and
  device-resident stage storage are absent until a concrete consumer contract
  requires them.
- `alloc` is required only to construct `StepWorkspace`; stepping and all
  policy reports remain allocation-free.
- Tableaus are sealed because coefficient correctness is provider-owned.
  Consumers request a new method upstream instead of supplying unchecked
  coefficients.

## Verification

- value-semantic tests exercise Euler, midpoint, and RK4 against independent
  analytical or polynomial results;
- a time-dependent system proves stage instants use the `C` coefficients;
- property tests cover constant derivatives and exact event clipping;
- negative tests cover invalid time, dimensions, controller parameters,
  ordering, and ratios;
- layout tests pin zero-sized marker and transparent-newtype claims;
- an instrumented global allocator records zero allocation activity across
  repeated post-construction steps;
- the crate builds without default features and its public doctest runs.
