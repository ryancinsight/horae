# Horae ownership gap audit

## Finding 2026-08-20: horae scope-vs-delivery audit

Static audit at detached head `a05dbeb`
(`a05dbeb` = merge of PR #23, `git status --porcelain` empty before the pass).
Read-only: no build, test, lint, or Git state-changing command was executed, so
nothing here asserts a passing suite. Every claim is a file-and-line citation
or the output of a counting command.

### Measured shape

- One package (`Cargo.toml` has no `[workspace]`), `no_std + alloc`
  (`src/lib.rs:8`, `:12`), edition 2024, `publish = false`.
- 1388 `src` LOC across 28 files; largest file `src/integration/stepper.rs`
  at 243 lines. 771 `tests` LOC across six integration binaries, 71
  `examples` LOC, no `benches/`.
- 23 `#[test]` functions in `tests/` (two of them proptest cases) plus one
  doctest in `README.md` included via `src/lib.rs:7`.
- Documentation: seven numbered book chapters plus an introduction and one
  example page (`docs/book/SUMMARY.md`); one ADR, status Accepted.

### Conformance floor

Clean at every measured class: zero `todo!`, zero `unimplemented!`, zero
TODO/FIXME/HACK, zero `#[allow]` sites, one justified `#[expect]`
(`examples/ordered_decay.rs:3`), zero production `unwrap()`, zero `dyn` in
`src`, zero `pub use ... as ...`, zero files over the 500-line target, zero
type-suffixed identifiers, and `lib.rs` plus every `mod.rs` manifest-only.
`missing_docs` and `unwrap_used` are denied and `unsafe_code` forbidden at the
manifest and crate level (`Cargo.toml` `[lints]`, `src/lib.rs:9`–`:10`).
A committed nextest budget exists (`.config/nextest.toml`: slow at 30s,
terminate after 60s) and the toolchain is pinned (`rust-toolchain.toml`).

### Capability coverage against the declared boundary

Every one of the seven capabilities the `README.md` Boundary section claims is
implemented with real, input-sensitive computation and no stub:
typed `Instant`/`StepSize` over Aequitas time, the borrowed-slice
`ExplicitSystem` seam, const-generic tableaus with one shared recurrence,
reusable stage workspaces, error-based adaptive reports, borrowed event
schedules with clipping, and zero-sized subcycle plans.

Integrator set: Euler, Midpoint, `Rk4`, and `DormandPrince` 5(4)
(`src/integration/tableau/methods.rs:9`, `:22`, `:35`, `:60`, `:107`). No
leapfrog/Verlet, SSP-RK, implicit, or IMEX surface is declared anywhere, and
ADR 0001 explicitly defers implicit integration until a second consumer
supplies a residual/Jacobian contract — so their absence is a recorded
boundary, not an undelivered claim. Likewise stability and CFL criteria are
declared out of scope (`README.md` Boundary; ADR 0001 "Move equations or CFL
policy into Horae"), so no typed CFL constraint is expected here.

The stepping recurrence is genuine: `evaluate_stages`
(`src/integration/stepper.rs:126`) consumes the tableau `A`/`C` coefficients
and the caller's system, and `combine_embedded_output` (`:194`) forms the
primary and embedded results from the shared stage derivatives.

### Verification depth — the material gap

The existing suite is strong on value semantics, negative paths, and derived
tolerances: every assertion cites a rounding-unit derivation rather than a
tuned epsilon (`tests/fixed_step.rs:93`, `tests/policy.rs:42`,
`tests/embedded.rs:78`), the Sterbenz and `gamma_4` boundary conditions are
pinned by tests (`tests/policy.rs:142`, `:213`), layout claims are proved
(`tests/layout.rs`), and an instrumented allocator proves zero allocation
across 16 steps (`tests/allocation.rs:70`–`:80`). Generic instantiation
coverage exists at `f32` and `f64` for the fixed-step methods
(`tests/fixed_step.rs:36`, `:102`) and the controller (`tests/policy.rs:56`).

What is missing is the independent oracle that matters most for an integrator:

1. No observed-order-of-accuracy study. The formal `ORDER` constants
   (`methods.rs:10`, `:23`, `:36`, `:61`) are consumed by
   `AdaptiveController::assess::<ORDER>` to form the step-scaling exponent
   (`src/adaptive/controller.rs:122`) but are never checked against a measured
   convergence rate. A grep for refinement or convergence across `tests/` and
   `examples/` returns only the two-point `h^5` ratio at `tests/embedded.rs:85`,
   which measures the *embedded error estimate*, not the order of the
   integrated solution. Filed as H-ORDER-001.
2. No closed adaptive loop. All eight `assess` call sites in the tree
   (`tests/policy.rs:38`, `:47`, `:62`, `:69`, `:76`, `:93`, `:97`, `:103`;
   `examples/ordered_decay.rs:57`) supply hand-written error and scale values.
   No test feeds `step_embedded_into`'s error-estimate slice into the
   controller, and no test exercises reject → re-step at the suggested size →
   accept. Filed as H-ADAPT-LOOP-002.
3. No multi-step accuracy composition. The only stepping loop in the tree
   (`tests/allocation.rs:44`) asserts allocation counts; every accuracy
   assertion in the repository is single-step. Filed as H-MULTISTEP-003.
4. `DormandPrince` and `step_embedded_into` are `f64`-only in tests
   (`tests/embedded.rs`), unlike the fixed-step and controller suites. Filed as
   H-DP-SCALAR-004.

Adaptive policy itself is real, not a mock: mixed absolute-relative
normalization, order-derived scaling, min/max clamping, and typed errors for
non-finite observation, overflowing tolerance, invalid order, and invalid
suggested step (`src/adaptive/controller.rs:96`–`:128`), with the accept,
reject, invalid-parameter, and non-finite branches all covered. Event clipping
is verified at exact event times, including duplicate-skip, no-clip-without-
crossing, and a high-magnitude case (`tests/properties.rs:60`, `:87`;
`tests/policy.rs:109`, `:161`).

### Documentation drift found

- Fixed in this pass: `README.md:164` said behavioral tests compare "all three
  method instantiations" — four tableaus now carry oracle-based tests.
  `docs/book/rk4.md:38` said "Horae provides three sealed zero-sized method
  markers"; corrected to four with the chapter's scope stated and a link to
  `rk5.md`.
- Filed, not fixed (each needs a judgement call or a citation):
  ADR 0001's Consequences section still lists "Adaptive embedded-pair
  estimation" as absent though `DormandPrince` and `step_embedded_into` ship
  (H-ADR-STALE-006); `docs/adr/README.md:3` instructs regeneration through
  `scripts/adr-index.py`, which does not exist in this repository
  (H-ADRIDX-007); `docs/book/rk4.md:226` asserts RK4 stability is
  "approximately 2.8 times CFL compared to forward Euler", an unsourced figure
  that reads as a ratio while approximating RK4's own real-axis boundary,
  inside a repository that disclaims CFL ownership (H-STAB-DOC-005);
  `src/integration/mod.rs:1` still summarizes the module as "Fixed-step
  explicit integration" while re-exporting the embedded surface
  (H-MODDOC-010).

### CI and supply-chain observations

`.github/workflows/ci.yml` pins every action to a full SHA and declares
least-privilege `permissions`, and the `verify` job carries
`timeout-minutes: 30`. Two floor gaps: the `supply-chain` job (`:52`) carries
no `timeout-minutes`, and only one cargo invocation in the workflow (`:34`)
passes `--locked` — the feature check, clippy, nextest, doctest, doc, and
example steps re-resolve against the network rather than verifying the
committed lock (H-CI-LOCKED-008). Separately, `Cargo.toml:5` declares
`rust-version = "1.95"` while `rust-toolchain.toml` pins `1.97.0` and no job
builds at the declared floor, so the MSRV claim is unverified (H-MSRV-009).

### Performance observation

`DormandPrince::B` equals its `A[6]` row (`methods.rs:85`–`:103`) — the FSAL
property — but `evaluate_stages` (`stepper.rs:143`) evaluates all seven stages
on every call, so a continuing accepted step pays one avoidable right-hand-side
evaluation. The crate carries no benchmarks, so this is an observation from the
coefficient structure, not a measurement (H-FSAL-011).

### PM-record note

`backlog.md` and `checklist.md` record "Nextest 20/20" as the most recent
counted run (entries dated 2026-08-16 and 2026-08-19); the tree now contains 23
`#[test]` functions. No run was performed in this pass, so the current count is
unknown — the discrepancy is only noted, not resolved.

### Completeness

Approximately 84% of Horae's own declared scope is delivered and verified.
Denominator: the seven capability bullets in the `README.md` Boundary section,
the decisions recorded in Accepted ADR 0001, and the seven chapters listed in
`docs/book/SUMMARY.md`. Weighting per the audit rubric — capabilities
implemented without stubs 40% (full marks: all seven present, real, and
stub-free), verification depth 25% (roughly half: exemplary value-semantic and
derived-bound coverage, but no order-of-accuracy refinement study, no closed
adaptive loop, and no multi-step accuracy test), documentation 20% (most marks:
complete book and README with four filed stale claims), conformance floor 15%
(near-full: clean at every measured class, docked for the two CI gaps and the
unverified MSRV).


## HORAE-LOCKSTEP-075 — refreshed consumer lock hosted closure 2026-08-19

The standalone Horae lock now follows the current merged Atlas provider heads:

- Aequitas: `260ad10dd5480eef8c82958d1d148199656db59e`
- Eunomia: `85e590b789505c66f5174043c2e7e851c20547a5`

Only `Cargo.lock` changed. Local-graph format, locked metadata, all-feature
check, no-default-features check, warning-denied Clippy, Nextest 20/20,
doctest 1/1, and rustdoc pass while the Atlas overlay resolves the refreshed
provider graph. Cargo-deny passes advisories, bans, licenses, and sources;
its only output is the existing unmatched-source warning for the two Git
origins. After restoring the committed standalone lock, root-overlay
`cargo nextest --locked` refuses before test execution because local patches
are not represented in the lock. Exact standalone locked closure remains the
hosted PR gate. Commit `9cc9fd8` is merged through PR #19 at default
`1ed6a17`; post-merge CI `32202560133` passes verification and supply-chain,
and Pages deployment `32202559349` passes. The provider lockstep item is
closed; parent-gitlink integration remains an Atlas-owned step.

## HORAE-LOCKSTEP-074 — Aequitas/Eunomia consumer pin — local closure 2026-08-16

Horae commit `65cb253` advances its standalone lockfile to the fetched
provider defaults without changing the manifest boundary:

- Aequitas: `5114cd12b6d7769628f2ebff83e26f9c5e19caed`
- Eunomia: `88c685f2a733df73f753f2351b2e4d2ede8ff478`

The provider branch passes `cargo fmt --all -- --check`, locked all-feature
build, locked no-default-features check, strict all-target Clippy, Nextest
(20/20), doctests (1/1), warning-denied rustdoc, the seven-chapter
`rustup run 1.97.0 mdbook test docs/book` suite, the `ordered_decay` example,
and cargo-deny advisories, bans, licenses, and sources. The book test was run
with the exact locked artifacts on the shared target after removing only the
verified Horae/Aequitas/Eunomia derived package outputs; no source or peer
checkout files were removed.

The Atlas parent overlay is intentionally excluded from this standalone
lockfile verification: running Cargo from `D:\atlas` injects local first-party
patches and requests overlay-only lock records. At the local-closure point,
hosted integration was pending; the merged-default evidence follows.

PR #16 merged at Horae default `379f506`. Post-merge CI run `31964298702`
passes the provider verification and supply-chain jobs, and Pages deployment
`31964297946` passes. The remaining cross-repository action is advancing the
parent Atlas Horae gitlink to this merged default.

## ATLAS-HORAE-AUDIT-073 — Isolated provider re-verification — closed 2026-08-16

The current provider default `1068651` was re-verified from outside the Atlas
umbrella directory so the parent `.cargo/config.toml` development overlay could
not alter the standalone lockfile. The exact locked commands were:

- `cargo fmt --check --manifest-path D:\atlas\worktrees\horae-audit-20260816\Cargo.toml`
- `cargo check --locked --all-features --all-targets --manifest-path D:\atlas\repos\horae\Cargo.toml`
- `cargo clippy --locked --all-targets --all-features --manifest-path D:\atlas\repos\horae\Cargo.toml -- -D warnings`
- `cargo nextest run --locked --all-features --manifest-path D:\atlas\repos\horae\Cargo.toml` — 20/20 passed
- `cargo test --locked --doc --all-features --manifest-path D:\atlas\repos\horae\Cargo.toml` — 1 passed
- `cargo doc --locked --no-deps --all-features --manifest-path D:\atlas\repos\horae\Cargo.toml`

The standalone checkout remains clean after verification. Running the same
locked commands from inside `D:\atlas` is a separate environment: the parent
overlay injects local first-party patches that are not represented in Horae's
standalone lockfile, so Cargo refuses the command before compilation and asks
to rewrite `Cargo.lock`. That is an umbrella verification/configuration
boundary, not evidence that the provider lockfile should be changed. The
provider lockfile remains the standalone and hosted source of truth.

These checks establish formatting, compilation, static diagnostics, test,
doctest, and documentation behavior only. They make no new runtime,
performance, memory, or hardware-backend claim. No source-level placeholder
markers were found, and the remaining H-001 through H-003 items below retain
their consumer- or owner-gated status.

## ATLAS-HORAE-EXACTNESS-069 — resolved 2026-08-14

The event contract now treats `EventClip::event()` as the authoritative
boundary and states the Sterbenz precondition required for exact reconstruction
from `start + step`. A regression at `start = 1e8` and a `1e-6` event offset
passes under that precondition. The subcycle contract no longer claims
bit-identical parent reconstruction for every ratio; the ratio-three `f64`
policy documents and tests the derived `gamma_4` bound for its four rounded
operations. The root Atlas backlog owns the cross-repository integration
record.

The generated ADR index now records ADR 0001's existing `Accepted` status;
the index is derived state and no decision content changed.

## Boundary

Horae owns typed simulation time, explicit tableaus, adaptive control, event
scheduling, and subcycle policy. It does not own equation formulation,
stiff/implicit stepping, or consumer solver policy.

## Provider verification (2026-08-12)

- Strict all-target check with `-D warnings`: pass.
- Clippy `-D warnings` (all-targets/all-features): pass.
- Nextest (all-features): 16/16 pass, 0 skipped.
- Doctests: pass.
- Hermetic mdBook Rust examples: pass for all six chapters in a staged
  dependency view; the clean-checkout CI gate is now committed.
- `--no-default-features` check: pass.
- Local planning trail (backlog/checklist/gap_audit) authored at this audit.

## ATLAS-HORAE-LOCK-070 — resolved locally 2026-08-14

The committed `Cargo.lock` contained 55 `[[patch.unused]]` records emitted by
the Atlas umbrella overlay. Those records are not valid standalone lock state:
the hosted `cargo build --all-features --locked` gate rejected the lock before
book tests ran. The lock now has zero overlay records and records the Git
sources for Aequitas and Eunomia directly. Standalone offline
`cargo metadata --locked --no-deps` passes. Hosted verify run `31859557127`
passes the locked build and six-chapter book suite; Pages book build
`31859557311` also passes.

## ATLAS-HORAE-TOOLCHAIN-071 — resolved 2026-08-14

The hosted run after `ATLAS-HORAE-LOCK-070` passed the locked build but failed
the book test with E0514: Cargo produced Horae, Aequitas, and Eunomia artifacts
with Rust 1.97 while mdBook invoked Rust 1.95 rustdoc. The CI book step now
derives the active override from `rustup show active-toolchain` and runs both
Cargo and mdBook through `rustup run`, keeping compilation and doctest tooling
on one pinned compiler. Verify run `31859557127` passes the full workflow,
including book tests, doctests, rustdoc, and the example.

## H-ORDER-002 — Stage-time coefficient refinement — in progress

The autonomous `y' = y` refinement fixture does not read the stage-time
argument, so it cannot detect a regression in a tableau's `C` coefficients.
The replacement audit adds the non-autonomous exact solution of `y' = t + y`,
`y(0) = 1`, whose endpoint at one is `2e - 2`. The generic f64 refinement
oracle classifies Euler (1), Midpoint (2), RK4 (4), and Dormand–Prince (5)
through the real stepping path with no tunable decimal tolerance. The focused
fixed-step Nextest binary passes 6/6 and warning-denied all-target Clippy
passes; doctests and Rustdoc remain to be collected after the shared build
lock drains.

## H-ORDER-001 — Observed tableau order by refinement — closed 2026-08-20

The formal orders are now exercised through the real stepping path rather than
only read from tableau constants. The test integrates `y' = y` from zero to
one at dyadic step counts and compares the numerical state with `e`; the
coarse/fine error ratio is classified by its nearest formal order. The f64
oracle covers Euler (1), Midpoint (2), RK4 (4), and Dormand–Prince (5). The
f32 oracle covers the non-embedded tableaus at a coarser range where their
truncation signal remains above the rounding floor; the existing embedded
tests retain f32/f64 Dormand–Prince execution coverage.

The exact-lane source mutation of the RK4 output weight fails the f64
refinement test. Format, locked all-target check, warning-denied Clippy,
all-feature Nextest (25/25), doctest (1/1), and warning-denied Rustdoc pass at
the current lane revision. The full locked all-featured Nextest suite passes
26/26, no-default-features Cargo check passes, warning-denied all-target Clippy
passes, the doctest gate passes 1/1, and warning-denied Rustdoc generates
successfully. Hosted PR and Pages evidence remain pending. No stability or
implicit-integration claim is made.

## Gap inventory

| ID | Description | Status |
|----|-------------|--------|
| H-ORDER-002 | Stage-time coefficient refinement | In progress — non-autonomous exact solution added; doctest/Rustdoc and hosted evidence pending |
| H-ORDER-001 | Observed order of accuracy by refinement | Closed — closed-form f64 oracle plus f32 non-embedded coverage |
| H-001 | Higher-order embedded adaptive tableaus (Dormand-Prince family) | Closed — `DormandPrince` and `step_embedded_into` provide a shared seven-stage 5(4) pair |
| H-002 | Stiff/implicit integration policy | Open — explicit-policy-only boundary today |
| H-003 | Registry publication (`publish = false`, occupied name) | Open — owner-gated facade/publication follow-up |

No delivered source-level gaps remain: no `TODO`/`FIXME`/`unimplemented!`
markers exist in `src/`, and the Rust gates are green at the audited revision.
H-004 is closed: Rust examples are standalone through hidden setup, prose
formulas and diagrams use non-Rust fences, and CI builds the locked dependency
set before running the full book test.
