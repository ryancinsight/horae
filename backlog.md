# Horae backlog

## ATLAS-HORAE-WORKFLOW-PIN-2026-08-20 — Refresh shared book workflow pin [patch] — hosted verification pending

- Owner: Atlas integration. Scope is the Pages caller and this PM/changelog
  record; the Horae implementation and lockfile remain unchanged.
- Acceptance: pin the caller to the current Atlas reusable workflow while
  preserving the existing executable book-test inputs, then pass the exact
  hosted source and Pages gates before advancing the Atlas gitlink.

Strategic roadmap; tags `[patch]`/`[minor]`/`[major]`/`[arch]` per SemVer class.
Horae is the Atlas typed-simulation-time and integration-policy SSOT: typed
time/duration values, explicit RK-family tableaus, adaptive step control,
event scheduling, and subcycle policy. The current production consumers use
typed time/subcycling in Harmonia and validated step sizes in Helios; the
CFDrs and Kwavers stepping migration remains a consumer-owned integration
item.

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
- [x] [patch] Make the provider book test hermetic (H-004): every Rust example
  has standalone hidden setup, non-Rust fences are typed as text, and CI runs
  `mdbook test docs/book` after building the locked all-feature dependency set.
  Evidence: the complete six-chapter book suite passes locally in a staged
  dependency view.
- [x] [patch] Remove Atlas development-overlay `[[patch.unused]]` records from
  the committed standalone `Cargo.lock`. Standalone offline metadata resolves
  the Aequitas and Eunomia Git sources with `--locked`; the lock no longer
  carries umbrella-only patch state that invalidates the hosted locked book
  build. The exact hosted verification is tracked by PR #14.

## Gate evidence (2026-08-12 foundation audit)

- [x] [patch] Canonical provider gates at the audited revision: strict
  all-target check with `-D warnings`, Clippy `-D warnings`
  (all-targets/all-features), Nextest 16/16 (all-features), doctests, and
  `--no-default-features` check all pass. See the Atlas root
  `ATLAS-FOUNDATION-PLANNING-002` entry for the verified matrix.

## Deferred (documented boundary)

- [x] [minor] Higher-order adaptive tableaus: `DormandPrince` provides a
  seven-stage fifth/fourth-order embedded pair through the shared recurrence;
  `step_embedded_into` writes the primary result and local-error estimate
  without allocation. Evidence: polynomial exactness, embedded-error scaling,
  dimension-negative, doctest, and provider gates.
- [x] [patch] Audit the stiff/implicit integration boundary (2026-08-19).
  The current production search finds no consumer implementation of
  `ExplicitSystem` and no consumer call to `step_into` or
  `step_embedded_into`; Harmonia uses Horae's typed time/subcycle contracts
  and Helios uses `StepSize`. Athena's roadmap requires a second concrete
  residual/Jacobian consumer before a shared nonlinear policy is added.
  ADR 0001 therefore remains correct: no implicit solver is added without
  that contract.
- [ ] [patch] Add an implicit integration policy only after a second concrete
  consumer supplies a residual/Jacobian contract, analytical oracle, and
  bounded convergence tests.
- [ ] [patch] Registry publication remains `publish = false` (occupied name);
  facade/publication work is an owner-gated follow-up.

## HORAE-LOCKSTEP-074 — Aequitas/Eunomia consumer pin [patch] — closed 2026-08-16

- Owner: current Atlas session; scope: Horae's `Cargo.lock` and provider-local
  dependency-coherence records. No Horae source or peer checkout files are in
  scope.
- Acceptance: the locked Aequitas and Eunomia revisions equal the fetched
  provider defaults `5114cd1` and `88c685f`, while the manifest remains
  Git-sourced with its existing semver requirements. The locked provider
  compile, tests, lint, documentation, and supply-chain gates must pass.
- No path dependency, compatibility shim, or overlay-only lock state is
  permitted.
- Evidence: `65cb253` pins Aequitas at `5114cd12b6d7769628f2ebff83e26f9c5e19caed`
  and Eunomia at `88c685f2a733df73f753f2351b2e4d2ede8ff478`. From outside the
  Atlas overlay, formatting, locked build/check, strict Clippy, 20/20 Nextest,
  1/1 doctest, warning-free rustdoc, seven-chapter `mdbook test`,
  `ordered_decay`, and cargo-deny all pass.
- Hosted evidence: PR #16 merged at default `379f506`; post-merge CI
  `31964298702` passes verify and supply-chain, and Pages deployment
  `31964297946` passes. Atlas's Horae gitlink remains a separate integration
  step owned by the parent repository.

## HORAE-LOCKSTEP-075 — Refresh merged Aequitas/Eunomia lock heads [patch] — closed 2026-08-19

- Owner: Atlas integration session; scope: `Cargo.lock` plus this provider's
  lock-coherence evidence. No Horae source or manifest change is in scope.
- The lock now records Aequitas `260ad10dd5480eef8c82958d1d148199656db59e`
  and Eunomia `85e590b789505c66f5174043c2e7e851c20547a5`, the current merged
  provider defaults used by Atlas. The Git-sourced dependency boundary is
  unchanged and no overlay-only records or compatibility paths were added.
- Local-graph evidence: format, locked metadata, all-feature check,
  no-default-features check, warning-denied Clippy, Nextest 20/20, doctest
  1/1, and rustdoc pass while the Atlas overlay resolves the refreshed local
  provider graph. `cargo deny check` passes advisories, bans, licenses, and
  sources; it reports only the existing unmatched-source warnings for the two
  Git origins.
- Evidence limit: after restoring the committed standalone lock form, root
  overlay `cargo nextest --locked --all-features` refuses before test
  execution because local patches are not represented in the lock. The exact
  standalone locked closure is therefore the hosted PR gate, not the local
  overlay run.
- Commit `9cc9fd8` is pushed on `codex/horae-lockstep-075` and merged through
  PR #19 at default `1ed6a17`. Post-merge CI run `32202560133` passes the
  provider verification and supply-chain jobs; Pages deployment
  `32202559349` passes. The Atlas parent may consume this exact default head.
