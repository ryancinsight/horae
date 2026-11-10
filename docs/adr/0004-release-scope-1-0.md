# ADR 0004: Horae release scope for 1.0

- Status: Accepted
- Date: 2026-11-09
- Class: `[arch]` `[minor]`

## Context

Horae (`horae-time` 0.2.2) provides typed simulation time and integration
policy for the Atlas stack. The backlog contains blocked provider-boundary and
correctness-coherence items (`H-PROVIDER-RANGE-20261019`,
`H-ADAPT-ORDER-RANGE-001`, `H-PROVIDER-BOUNDARY-001`) and in-progress
verification work (`H-ORDER-002`). A milestone ladder is required to order work
to 1.0.

## Decision

Adopt a three-milestone ladder to 1.0 recorded in `backlog/milestones.md`:

- **M-coherence** (0.2.3): resolve provider-boundary/range and open correctness
  or verification items with landed, independently reviewable evidence.
- **M-stability** (0.3.0): deliver stable integration contracts and consumer
  coherence against current stack providers with full gates.
- **M-1.0** (1.0.0): release-ready (authority-gated): documented API, examples,
  semver and package gates passing, and no open correctness items.

## Alternatives

- Pursue 1.0 without closing current blocked provider-boundary items: rejected
  because consumer coherence and release acceptance remain unresolved.
- Leave planning implicit in backlog prose: rejected because stack policy
  requires runnable milestone exit checks.

## Consequences

- Open backlog items carry `milestone:` assignments aligned to the ladder.
- Milestones close only when exit checks pass on fetched default commits;
  publication/release remains authority-gated.
- This ADR is rewritten in place as scope evidence changes.
