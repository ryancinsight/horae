# Horae backlog

## H-ORDER-002 — Verify stage-time coefficients `verification` `patch` — in progress

- Owner: Atlas integration; scope is the non-autonomous refinement oracle in
  `tests/fixed_step.rs` and synchronized provider records. No tableau or
  consumer implementation changes are in scope.
- Finding: the H-ORDER-001 autonomous fixture ignores the stage-time input, so
  a regression in a tableau's `C` coefficients can pass its convergence test.
- Acceptance: integrate the exact solution of `y' = t + y`, `y(0) = 1`, at
  dyadic refinements; classify Euler, Midpoint, RK4, and Dormand–Prince by
  their declared orders without a tunable decimal tolerance; run the locked
  format, warning-denied Clippy, Nextest, doctest, and Rustdoc gates.
- Current evidence: the new time-dependent test passes with the other five
  fixed-step tests (6/6); the full locked all-featured Nextest run passes
  26/26, no-default-features Cargo check passes, warning-denied all-target
  Clippy passes, the doctest gate passes 1/1, and warning-denied Rustdoc
  generates successfully. Hosted PR and Pages evidence remain pending.

## H-ORDER-001 — Verify observed order of accuracy by refinement `verification` `patch` — done 2026-08-20

- Owner: current Atlas session; scope is the closed-form refinement oracle in
  `tests/fixed_step.rs`. No tableau, API, consumer, or dirty-primary changes
  are included.
- Acceptance met: the exact solution of `y' = y` is integrated from zero to
  one at dyadic refinements. The observed `log2(e(h)/e(h/2))` rounds to the
  declared order for Euler, Midpoint, RK4, and Dormand–Prince in f64. The
  generic f32 path verifies the non-embedded tableaus at a coarser signal
  range; existing embedded tests retain f32/f64 Dormand–Prince execution
  coverage because its fifth-order f32 truncation signal is roundoff-limited
  at this finite range.
- Evidence: clean lane `fix/horae-order-oracle` is based on fetched
  `origin/main` `a05dbeb`. The source-level mutation of the RK4 output weight
  fails the f64 refinement oracle and is restored. Exact provider gates pass:
  format, locked all-target check, warning-denied Clippy, all-feature Nextest
  25/25, doctest 1/1, and warning-denied Rustdoc. The no-default matrix is
  recorded in the delivery closeout.

## ATLAS-HORAE-WORKFLOW-PIN-2026-08-20 — Refresh shared book workflow pin [patch] — done 2026-08-20

- Owner: Atlas integration. Scope is the Pages caller and this PM/changelog
  record; the Horae implementation and lockfile remain unchanged.
- Acceptance: pin the caller to the current Atlas reusable workflow while
  preserving the existing executable book-test inputs, then pass the exact
  hosted source and Pages gates before advancing the Atlas gitlink.
- Evidence: source `aaed0cff8e777d62fcaff4f20b3347bb1eefa403`, PR #22, and
  merged default `c2e7766847e3ef28125b809d98fe07250acc6cec`. Exact PR
  `verify`, `supply-chain`, and Pages book-build checks pass; post-merge CI
  `32341053963`, Deploy mdBook `32341054529`, and dynamic Pages `32341052968`
  pass. Live Pages returns HTTP 200 with the expected Horae title.
- Delivery: the Atlas gitlink records the merged default and this provider
  item is closed.

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

## Gap audit 2026-08-20 — scope-vs-delivery items

Filed by the Atlas gap-audit pass at detached head `a05dbeb`. Evidence is
recorded in `gap_audit.md` under
`Finding 2026-08-20: horae scope-vs-delivery audit`. No item below is claimed
verified; each carries its own acceptance oracle.

### H-ORDER-001 — Verify observed order of accuracy by refinement [patch] — status: done 2026-09-02

- Outcome: each sealed tableau's declared `ORDER` is verified against a
  measured convergence rate, so the constant consumed by
  `AdaptiveController::assess::<ORDER>` (`src/adaptive/controller.rs:122`) is
  evidence-backed rather than declared.
- Scope: `tests/`. Non-goals: no source, tableau, or API change; no stability
  or CFL criteria (ADR 0001 excludes them).
- Acceptance oracle: for Euler, Midpoint, `Rk4`, and `DormandPrince`, a
  timestep-refinement study over a smooth initial-value problem with a
  closed-form solution recovers `log2(e(h)/e(h/2))` within a derived tolerance
  of the formal order (1, 2, 4, 5), with the refinement range chosen so
  truncation error dominates rounding and the tolerance derived from that
  separation, not tuned.
- Dependencies: none. Risk/change class: [verification] [patch]. Effort M.
- Note: `tests/embedded.rs:85` establishes the `h^5` scaling of the *embedded
  error estimate* only; no study covers the order of the integrated solution.
- **Delivered 2026-09-02** as `tests/order_of_accuracy.rs`: a four-level
  refinement study on `y' = −y` over `t ∈ [0, 1]`, one clause per tableau.
  Every declared `ORDER` is recovered — Euler 1, Midpoint 2, `Rk4` 4,
  `DormandPrince` 5.
- **The tolerance is derived from the run, not chosen.** The two-level
  estimate `log2(e(h)/e(h/2))` carries a defect `K·h + M·h² + …`; Richardson
  on that estimate cancels the linear term, four levels give two extrapolants,
  and the distance between them measures what the finer one still neglects.
  That distance plus the rounding floor propagated through the same logarithms
  is the bound. A separate clause asserts the raw pair estimates are
  converging, so the extrapolation cannot be arithmetic over a diverging
  sequence, and each level is asserted clear of the rounding floor by 100x so
  the study cannot measure the floor instead of the method.
- Evidence that the clauses bite: declaring `Rk4::ORDER = 3` fails
  `rk4_recovers_its_declared_fourth_order` and leaves the other three passing.

### H-ADAPT-LOOP-002 — Close the adaptive control loop in a test [patch] — status: done 2026-09-02

- Outcome: the adaptive controller is verified against a real embedded local
  error estimate rather than synthetic scalars.
- Scope: `tests/`. Non-goals: no controller API change.
- Acceptance oracle: a test drives `step_embedded_into` on a system with a
  closed-form solution, feeds the resulting error-estimate slice and state
  scale into `AdaptiveController::assess::<5>`, asserts a `Reject` at a step
  that exceeds tolerance, re-steps at the suggested step, asserts `Accept`,
  and asserts the accepted state against the closed-form oracle within a
  derived bound.
- Dependencies: none. Risk/change class: [verification] [patch]. Effort M.
- Note: all eight current `assess` call sites (`tests/policy.rs:38`–`:103`,
  `examples/ordered_decay.rs:57`) pass hand-written error values; the
  estimator-to-controller composition is untested.
- **Delivered 2026-09-02** as `tests/adaptive_loop.rs`. A real
  `step_embedded_into` estimate on `y' = -y` drives `assess::<5>`: the coarse
  step is rejected, each proposal is asserted strictly smaller, the loop runs
  to `Accept`, and the accepted state is asserted against `e^(-h)` within twice
  the local error the controller just certified — the estimate is accurate to
  the next order, so that is a sound bound rather than a chosen one.
- **A first attempt asserted the wrong contract** and is worth recording: it
  expected one re-step to clear tolerance. The controller clamps its scale
  factor, so from a large overshoot it takes several rejections — the test was
  wrong, not the controller. The rejection count is now bounded by the clamp
  (`0.2^5` per rejection against the initial normalized error), which is what
  proves the controller drives itself to a decision instead of stalling.
- A second clause pins determinism by bit equality: the same step on the same
  state must produce the same estimate and the same decision.
- Evidence the clauses bite: a controller patched to propose the step it just
  rejected fails the loop clause and leaves the determinism clause passing.

### H-MULTISTEP-003 — Value-semantic multi-step accumulation test [patch] — status: done 2026-09-02

- Outcome: repeated stepping is verified for accumulated state, not only for
  allocation behavior.
- Scope: `tests/`. Non-goals: no source change.
- Acceptance oracle: an N-step march over a closed-form system asserts the
  final state within a derived accumulated-error bound and asserts the final
  `StepReport::end()` against the analytic end instant.
- Dependencies: none. Risk/change class: [verification] [patch]. Effort S.
- Note: the only multi-step loop is `tests/allocation.rs:44`, whose assertions
  are allocation counts.
- **Delivered 2026-09-02** as `tests/multi_step_march.rs`. A 64-step `Rk4`
  march over `y' = -y` on `[0, 1]` asserts the final state against `e^(-1)`
  within `10·h^4/120` — the fourth-order local truncation constant
  accumulated over `1/h` steps, carrying an order of magnitude rather than a
  fitted factor — and the reported `StepReport::end()` against the analytic
  end instant within `STEPS · eps`. A companion clause asserts the deviation
  is nonzero, so the bound cannot pass vacuously on a march that is not
  integrating.
- Two further clauses: splitting a 32-step march into two 16-step marches
  must give the bit-identical result (the stepper carries nothing between
  calls beyond what it is handed), and a workspace/state dimension mismatch
  must be reported by name rather than absorbed.

### H-DP-SCALAR-004 — Instantiate the embedded pair for `f32` [patch] — status: done 2026-09-02

- Outcome: `DormandPrince` and `step_embedded_into` are covered at every
  shipped scalar, matching the generic coverage already present for the
  fixed-step methods and the controller.
- Scope: `tests/embedded.rs`. Non-goals: no new scalar type.
- Acceptance oracle: the polynomial-oracle and error-scaling assertions run
  from a generic helper instantiated at `f32` and `f64`, with per-scalar
  derived bounds.
- Dependencies: none. Risk/change class: [verification] [patch]. Effort S.
- Note: `tests/fixed_step.rs:36` already uses this generic-helper shape.
- **Delivered 2026-09-02.** `FourthDegreeRate` and `run` are generic over the
  scalar, and the polynomial-oracle and error-scaling assertions moved into
  `assert_polynomial_oracle_and_error_scaling<T>`, instantiated at `f64` and
  `f32`. The bounds are stated in units of `T::EPSILON` rather than as
  absolute constants, so the same derivation holds at both widths and only
  the epsilon it multiplies changes — which is what instantiating the second
  scalar is for.
- Evidence the clauses bite at both widths: expecting a `2^5` ratio of `31.9`
  instead of `32.0` fails the test.

### H-STAB-DOC-005 — Remove or source the book stability constants [patch] — status: todo

- Outcome: no unsourced numerical stability claim remains in a repository that
  declares stability and CFL criteria out of scope (`README.md` Boundary,
  ADR 0001 "Move equations or CFL policy into Horae").
- Scope: `docs/book/rk4.md`. Non-goals: no stability API.
- Acceptance oracle: `docs/book/rk4.md:226` ("approximately 2.8 times CFL
  compared to forward Euler") and `:161` ("moderately larger than Euler") are
  either deleted or replaced with a cited real-axis stability-interval
  statement carrying its source; `mdbook test docs/book` still passes.
- Dependencies: none. Risk/change class: [docs] [patch]. Effort S.
- Note: the classical RK4 real-axis stability interval is approximately
  `[-2.785, 0]` and forward Euler's is `[-2, 0]`, so the stated figure is the
  RK4 boundary itself, not a ratio to Euler, while the sentence asserts a
  ratio. A correct replacement needs a citation, so this was not fixed in the
  audit pass.

### H-ADR-STALE-006 — Revise ADR 0001 for the delivered embedded pair [patch] — status: todo

- Outcome: the one current record of the time-integration boundary decision
  matches the delivered surface.
- Scope: `docs/adr/0001-time-integration-boundary.md`, `docs/adr/README.md`.
  Non-goals: no boundary change; the implicit and dense-output exclusions
  stand.
- Acceptance oracle: the Consequences sentence listing "Adaptive embedded-pair
  estimation, dense output, implicit integration, and device-resident stage
  storage" as absent is rewritten in place to record the delivered embedded
  pair, the Decision section names `EmbeddedExplicitTableau` and
  `step_embedded_into`, and a dated revision note records what changed and
  why; the ADR index still passes its check.
- Dependencies: none. Risk/change class: [docs] [patch]. Effort S.
- Note: contradicted by `src/integration/tableau/methods.rs:107` and
  `src/integration/stepper.rs:88`.

### H-ADRIDX-007 — Restore or retarget the ADR index generator [patch] — status: done 2026-09-02

- Outcome: the ADR index stated regeneration procedure is executable in this
  repository.
- Scope: `docs/adr/README.md` and, if the generator is adopted, a committed
  script plus its CI check. Non-goals: no ADR content change.
- Acceptance oracle: either the referenced generator exists and its `check`
  mode runs in CI, or the "do not hand-edit" header is replaced with the
  procedure that actually applies.
- Dependencies: none. Risk/change class: [pm-hygiene] [patch]. Effort S.
- Note: `docs/adr/README.md:3` names `scripts/adr-index.py`; the repository
  has no `scripts/` directory.
- **Closed 2026-09-02 on the merged tree.** The note is stale: the header now
  names atlas's generator, called through the shared reusable workflow
  (`.github/workflows/adr-index.yml` → `adr-index-guard.yml`, `strict: true`).
  The item's "commit a generator here" branch was the wrong one — atlas owns
  one generator for the fleet, so adopting it is what keeps the index from
  drifting per repo.

### H-CI-LOCKED-008 — Run CI cargo steps against the committed lock [patch] — status: done 2026-09-02

- Outcome: hosted gates verify the committed `Cargo.lock` instead of
  re-resolving, and every CI job carries a bounded wait.
- Scope: `.github/workflows/ci.yml`. Non-goals: no lock content change.
- Acceptance oracle: the feature-check, clippy, nextest, doctest, doc, and
  example steps pass with `--locked`; the `supply-chain` job declares
  `timeout-minutes`; both jobs pass on a hosted run at the exact revision.
- Dependencies: the standalone lock must stay overlay-free (already closed by
  the `[[patch.unused]]` removal item). Risk/change class: [verification]
  [patch]. Effort S.
- Note: only `.github/workflows/ci.yml:34` currently passes `--locked`; the
  `supply-chain` job at `:52` has no `timeout-minutes`.
- **Delivered 2026-09-02.** Feature-check, clippy, nextest, doctest, doc and
  example all carry `--locked`; `supply-chain` and the new MSRV job carry
  `timeout-minutes`. **`Swatinem/rust-cache` builds its key with
  `cargo metadata` run without `--locked`**, so on a drifted lockfile it
  rewrites the file and every `--locked` step after it would verify a lock
  that is not the committed one — the gate passing for the wrong reason. Each
  cache step is now followed by `git diff --exit-code -- Cargo.lock`.

### H-MSRV-009 — Verify or drop the declared MSRV [patch] — status: done 2026-09-02

- Outcome: the `rust-version` claim is either enforced by a run or removed.
- Scope: `.github/workflows/ci.yml`, `Cargo.toml`. Non-goals: no dependency
  change to reach an older floor.
- Acceptance oracle: a CI job builds and checks the crate on the declared
  floor toolchain and passes, or `rust-version` is raised to the pinned
  toolchain with the change recorded in the changelog.
- Dependencies: none. Risk/change class: [verification] [patch]. Effort S.
- Note: `Cargo.toml:5` declares `rust-version = "1.95"`; `rust-toolchain.toml`
  pins `1.97.0` and every CI step uses the pin, so the floor is never
  exercised.
- **Delivered 2026-09-02** as the `MSRV (1.95)` job, which builds
  `--locked --all-features --all-targets` at the floor. **The floor is
  requested through `RUSTUP_TOOLCHAIN`, not the setup action**: the committed
  `rust-toolchain.toml` pin outranks what the action selects, so the obvious
  spelling would have re-tested 1.97.0 and left the claim as unverified as
  before. The job prints `rustc --version` and fails if it is not the floor,
  so it cannot pass vacuously.
- The floor was verified locally before the job was written —
  `RUSTUP_TOOLCHAIN=1.95.0 cargo check --locked --all-features --all-targets`
  compiles horae, eunomia and aequitas clean — so this does not ship a red
  check.
- Also adopted atlas's shared workflow linter, which catches the class that
  would otherwise make a broken `ci.yml` register no check at all.

### H-MODDOC-010 — Correct the integration module summary [patch] — status: done 2026-09-02

- Outcome: the module doc describes the module's actual surface.
- Scope: `src/integration/mod.rs`. Non-goals: none.
- Acceptance oracle: the `//!` summary covers embedded as well as fixed-step
  stepping; `cargo doc` stays warning-clean.
- Dependencies: none. Risk/change class: [docs] [patch]. Effort S.
- Note: `src/integration/mod.rs:1` reads "Fixed-step explicit integration."
  while the module re-exports `step_embedded_into` and `EmbeddedOutputs`.
- **Delivered 2026-09-02.** The summary names both stepping forms and says what
  the embedded one writes, with intra-doc links to the two entry points.
  `cargo doc` stays warning-clean under `RUSTDOCFLAGS=-D warnings`.

### H-FSAL-011 — Record or exploit the Dormand--Prince FSAL property [patch] — status: done 2026-09-02

- Outcome: the per-step evaluation cost of the embedded pair is either reduced
  to six evaluations for a continuing accepted step or explicitly documented
  as seven by design.
- Scope: `src/integration/`, `docs/book/rk5.md`, `tests/`. Non-goals: no
  tableau coefficient change; no dense output.
- Acceptance oracle: either a stage-reuse path is added whose accepted-step
  evaluation count is six and whose result matches the current seven-stage
  result bit-for-bit on a fixture, or `rk5.md` states that FSAL reuse is not
  exploited and why. Benchmark evidence is required for the former.
- Dependencies: H-ADAPT-LOOP-002 supplies the loop the reuse applies to.
  Risk/change class: [perf] [patch]. Effort M.
- Note: `DormandPrince::B` equals `A[6]`
  (`src/integration/tableau/methods.rs:85`–`:103`), the FSAL condition;
  `evaluate_stages` (`src/integration/stepper.rs:143`) evaluates all seven
  stages every call.
- **Delivered 2026-09-02 by the documenting branch, with the property made
  executable.** `dormand_prince_satisfies_the_fsal_condition` asserts
  `A[6] == B` bit-for-bit and `C[6] == 1`, so `docs/book/rk5.md` cannot go on
  describing a tableau that no longer has the property; perturbing one
  coefficient of the final stage row fails it.
- **Why not the reuse branch.** `step_into` and `step_embedded_into` are pure
  functions of their arguments — `tests/multi_step_march.rs` asserts that by
  splitting a march and comparing bit-for-bit. A carried derivative is valid
  only for a step that begins where the previous one ended *and was accepted*,
  so passing it as a plain argument makes a silent wrong answer reachable from
  safe code after a rejection, a step-size change, or an event; passing it as a
  typed continuation is a public API change to the whole stepping surface. The
  item's own oracle requires benchmark evidence for that branch and none
  exists, so the cost is documented as seven by design with the measurement
  that would reopen it named.
