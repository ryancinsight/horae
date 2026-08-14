# Horae backlog

Strategic roadmap; tags `[patch]`/`[minor]`/`[major]`/`[arch]` per SemVer class.
Horae is the Atlas typed-simulation-time and integration-policy SSOT: typed
time/duration values, explicit RK-family tableaus, adaptive step control,
event scheduling, and subcycle policy, consumed by the integrators (kwavers,
CFDrs, helios) and the domain layer.

## Delivered — foundation (0.1.0)

- [x] [arch] Scaffold the provider: typed simulation time and duration values
  over Aequitas/Eunomia scalars, explicit integration tableaus, adaptive
  control, and event scheduling contracts. Decision:
  [ADR 0001](docs/adr/0001-time-integration-boundary.md).
  Evidence: value-semantic tests for time arithmetic, fixed-step policy, and
  layout invariants.

- [x] [minor] Fixed-step and explicit integration policy: RK4 tableau,
  ordered propagation, and failure-context preservation on shape/system
  errors. Evidence: `fixed_step` and `policy` suites.

- [x] [minor] Adaptive step control with a generic `f32`/`f64` adaptive-policy
  contract, error-estimate-driven step selection, and bounded refinement.
  Evidence: `adaptive` suite plus the f32 adaptive-policy contract landed in
  PR #7 (`docs(horae): Complete book chapter prose and add f32 adaptive-policy
  contract`).

- [x] [minor] Event scheduling semantics: `EventSchedule` with duplicate-skip
  and no-clip-without-crossing guarantees, ordered event application, and
  deterministic time-ordered dispatch. Evidence: `events` suite
  (`b6fa57f docs(horae): Pin EventSchedule duplicate-skip and no-clip-without-crossing semantics`).

- [x] [patch] Numerical boundary contracts: event identity is authoritative
  through `EventClip::event()`, reconstruction is qualified by the Sterbenz
  condition, and ratio-three subcycle reconstruction carries a derived
  `gamma_4` bound. Evidence: large-magnitude event regression and policy
  rounding-bound test.

- [x] [patch] Refresh the generated ADR index from ADR 0001's existing
  canonical status header; no decision content changes.

- [x] [minor] Subcycle policy: nested time refinement with a bounded
  subcycling budget and rounded parent/child time alignment. Evidence: `subcycling`
  suite.

- [x] [patch] Author and close the provider book (ATLAS-HORAE-PROVIDER-DOCS-001):
  adaptive, events, explicit systems, RK4, and subcycling chapters with
  runnable examples. Evidence: mdBook build and the f32 adaptive-policy
  contract merged as PR #7 (`4ce76ca` on `horae-book-rebase`).

## Gate evidence (2026-08-12 foundation audit)

- [x] [patch] Canonical provider gates at the audited revision: strict
  all-target check with `-D warnings`, Clippy `-D warnings`
  (all-targets/all-features), Nextest 16/16 (all-features), doctests, and
  `--no-default-features` check all pass. See the Atlas root
  `ATLAS-FOUNDATION-PLANNING-002` entry for the verified matrix.

## Deferred (documented boundary)

- [ ] [patch] Make `mdbook test docs/book` hermetic. Existing book snippets
  mix standalone pseudocode, prose formulas, and cross-snippet declarations;
  the current command fails before reaching the changed event/subcycle text.
  The Rust crate gates and `cargo test --doc` remain green.
- [ ] [minor] Higher-order adaptive tableaus beyond the RK4 family (Dormand-
  Prince, etc.). The current explicit/adaptive surface satisfies existing
  consumers; a consumer need for embedded higher-order pairs would open this.
- [ ] [patch] Stiff/implicit integration policy. Explicit-policy-only boundary
  today; revisit when a consumer requires implicit stepping.
- [ ] [patch] Registry publication remains `publish = false` (occupied name);
  facade/publication work is an owner-gated follow-up.
