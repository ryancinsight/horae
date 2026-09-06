# ADR 0002: Add an implicit-system seam for stiff kinetics

- Status: Proposed
- Date: 2026-09-04
- Class: [arch] [minor]

## Context

Prometheus Phase 0 (atlas ADR 0058) requires a stiff integration path. Its
oracle is the Robertson benchmark — a three-species chain with timescales
spanning roughly seven orders of magnitude, the published case a non-stiff
integrator "visibly fails". Horae's current tableau set is entirely explicit
(ADR 0001): Euler, explicit midpoint, classical RK4, and Dormand–Prince. An
explicit method's absolute-stability region bounds the step by the fastest
timescale, `h ~ 1/λ_max`, so a stiff network is not merely slow — the step
count is the condition spread, and round-off decouples accuracy from stability
long before the slow dynamics resolve.

The capability belongs in Horae, not Prometheus (upstream ownership). Horae
owns time-integration policy (ADR 0001); Prometheus owns the reaction network
and must not grow its own stiff stepper.

## Decision

Add an implicit one-step family beside the explicit one, with the same
ownership rules ADR 0001 establishes.

- **`ImplicitSystem<T>`** is the stiff-system seam. It is a supertrait of
  `ExplicitSystem<T>` (the right-hand side is already sufficient for a
  non-stiff stepper) adding one method, `jacobian(time, state, out)` that
  writes `∂f/∂y` into a caller-owned row-major buffer. A system that cannot
  supply `∂f/∂y` remains explicit-only and simply does not implement the
  trait.
- **`BackwardEuler`** is the first implicit tableau:
  `y_{n+1} = y_n + h·f(t_{n+1}, y_{n+1})`. It is A-stable, so its step is
  bounded by accuracy alone.
- One **`step_implicit_into`** recurrence mirrors `step_into`: it forms the
  residual `g(y) = y − y_n − h·f(t_{n+1}, y)`, assembles the Newton matrix
  `I − h·∂f/∂y`, and iterates a damped Newton solve from the explicit-step
  predictor. The initial guess is the explicit Euler point, so a non-stiff
  region costs one extra solve and the stiff region is where the iteration
  earns it.
- No allocation: the residual, Newton matrix factorization, and step scratch
  reuse `StepWorkspace` buffers or a dedicated implicit workspace.

## Consequences

- Prometheus P6's Robertson oracle becomes exercisable honestly — an A-stable
  method steps it at the slow timescale, and the benchmark binds the stiff
  path rather than being skipped or faked with a tiny explicit step.
- The seam is additive. Explicit consumers and tableaus are untouched.
- The new contract surface is the Jacobian. A Jacobian-free fallback
  (finite-difference or Krylov) is out of scope for the first tableau and is
  recorded as a follow-up item.
- `ImplicitSystem: ExplicitSystem` keeps the trait set coherent: there is no
  second system seam, only a stiff-capable extension of the existing one.

## Rejected alternatives

### A Jacobian-free (Krylov/GMRES) solver from the first commit

Rejected because it binds the first slice to a subspace iteration whose
convergence and preconditioning decisions are themselves a research surface.
An exact small Jacobian (reaction networks are few-species) is the honest first
step; the Krylov fallback arrives when a large stiff system justifies it.

### Move the stiff stepper into Prometheus

Rejected per upstream ownership: a bespoke stiff solver in the consumer forks
the integration policy Horae exists to own, and every later stiff consumer
re-derives it.

## References

- H. H. Robertson, "The solution of a set of reaction rate equations", in
  J. Walsh (ed.), *Numerical Analysis: An Introduction*, Academic Press, 1966.
- E. Hairer and G. Wanner, *Solving Ordinary Differential Equations II —
  Stiff and Differential-Algebraic Problems*, 2nd ed., Springer, 1996,
  Chapter IV (stability, W-methods, BDF).