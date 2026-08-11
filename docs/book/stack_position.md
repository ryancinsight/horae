# Position in the Atlas stack

Horae is the time-integration policy provider. Its public surface is the
small, backend-neutral seam between a domain equation and an execution layer:

```text
Aequitas  -> typed physical time
Eunomia   -> scalar and numeric laws
   \
    Horae -> explicit stepping, adaptive decisions, events, subcycle ratios
     |
Athena / Harmonia / other consumers -> equations, coupling, convergence
Leto / Hephaestus / schedulers       -> storage and execution
```

Horae consumes `aequitas::Time<T>` for `Instant` and `StepSize`, and uses
Eunomia's `FloatElement` boundary for scalar arithmetic. It does not define a
second units vocabulary or scalar abstraction. Its integration API uses
borrowed slices, leaving arrays and device buffers to consumers that can adapt
their own storage at the boundary.

The ownership split is deliberate:

- Horae owns explicit Runge--Kutta tableaus, reusable stage workspaces,
  adaptive accept/reject policy, exact event clipping, and compile-time
  subcycle ratios.
- Consumers own equations, physical stability/CFL policy, event payloads,
  coupling invocation, state commit/retry, and synchronization.
- Aequitas, Eunomia, Leto, and Hephaestus remain the respective providers for
  units, scalar law, CPU arrays, and GPU execution.

The crate remains `no_std + alloc` in both feature configurations. The default
`std` feature only propagates standard-library support to its provider
dependencies. Keeping Horae independent of a scheduler, array backend, or
domain package lets each consumer adopt the contracts without a reverse edge.

The Atlas integration records and consumer migrations are maintained in the
root planning documents. Provider-local release flags and cross-repository
lockfile/gitlink delivery are separate release work; this chapter describes the
stable ownership boundary rather than claiming registry publication.
