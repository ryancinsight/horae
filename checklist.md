# Horae checklist

Active sprint target: **0.3.1** private nonfinite-residual correction [patch],
stacked on reviewed 0.3.0. Preserve provider ownership and publication holds.

## H-IMPLICIT-NONFINITE-20261020 [patch]

- [x] Isolate the reviewed `bc52f7d` source without editing the time-progress branch.
- [x] Reproduce false NaN success through real decay/exchange equations before production edits.
- [x] Require all residual components finite before the norm can certify convergence.
- [x] Preserve the existing error surface, tolerance policy, dense solve and iteration bound.
- [x] Pass the 55-test native suite, strict Clippy, no-default all-targets and Rustdoc.
- [x] Preserve every old test leaf and keep the unchecked final correction separately open.
- [ ] Complete independent exact-head review and any later public acceptance; local receipts accompany the private artifact.

## H-TIME-PROGRESS-20261020 [minor]

- [x] Inspect fresh instructions, PRs, ownership, workflow triggers and deployment settings.
- [x] Preserve the owned stage-time oracle and all existing test leaves/gates.
- [x] Reproduce stalled time and false successful stepping with unchanged regression tests.
- [x] Require strictly later finite endpoints and validate before explicit state work.
- [x] Cover native/wrapped scalars, all methods, event clipping and valid rounding boundaries.
- [x] Reproduce and record separate implicit nonfinite/final-iteration defects as open.
- [x] Complete native, lint, documentation, boundary, API and package checks; retain exact-head receipts.
- [x] Obtain independent source/API review and preserve the raw report; deliver exact-head confirmation with the private receipt.
- [ ] Renew public-action authorization and deployment/release scope before any later push.


## H-PROVIDER-RANGE-20261019 [patch] — target0.2.2

- [x] Retain current0.3/0.10 support and reviewed0.2.1 numerical source.
- [x] Verify current and future typed values,strict/API/MSRV and unchanged policy.
- [ ] Complete exact-head package/review/CI lifecycle;preserve0.2.1 local publication hold.


## Integration renewal — 2026-10-17 [patch]

PR #101 integrates the coherent 0.2.0 provider tree at
[accepted main](https://github.com/ryancinsight/horae/commit/ab42e5707a3b9a3cc080cfbbb8aa8e36bd4fc56f) with the independently reviewed
tree, passing main CI/ADR and automatic Pages. The pending publisher selects
nothing because horae-time remains unpublished; no publication tag exists.
The isolated invalid-order rebase preserves its reviewed production fix and
regression leaf byte for byte, keeps all current main provider/registry pins,
and reserves 0.2.1 as the next patch after the frozen 0.2.0 first-publication
candidate. This does not claim 0.2.0 is published. The original checkout and
first-publication handoff remain unchanged. Fresh exact-head verification and
independent review are renewed separately; acceptance stays held until the
reviewed 0.2.0 publication is verified. Historical 0.1.1 evidence below belongs
to the original old-provider head and does not certify this rebased head.


## H-ADAPT-ORDER-RANGE-001 — Invalid const order [patch]

- [x] Inspect exact main, current tasks and complete open-PR file ownership; preserve H-ORDER-002 and provider lanes.
- [x] Reproduce E0080 with the expanded boundary regression before editing production source.
- [x] Guard const coefficient derivation without changing the supported range, valid formula or observation-error precedence.
- [x] Test zero, maximum valid, neighboring unsupported and maximum usize orders for native/wrapped f32/f64.
- [x] Pass 45 native tests with zero skips, all configured local native/documentation/MSRV checks and current supply-chain policy.
- [x] Compare 12,544 frozen baseline/current result records at each of O0/O2/O3 exactly; preserve all external lock records and existing test leaves.
- [x] Assess committed locked-source API (223 pass; 30 inapplicable), package/dry-run and typed normalized consumers; normalized 45-test suite, two doctests and no-default check pass.
- [x] Reproduce the same fresh Aequitas ^0.2.1/main-0.3.0 resolution failure on unmodified main; preserve provider PR ownership.
- [ ] Assess final hosted gates after draft creation; the fresh Git graph remains blocked.
- [ ] Obtain exact-head independent review and explicit release acceptance; no merge/publication is performed here.

## H-PROVIDER-BOUNDARY-001 — Current provider and public API boundary

- [x] Verify exact #99 tree and preserve #99/#100 branches and current review.
- [x] Reproduce fresh Mnemosyne resolution failure and old quantity/scalar consumer failures before the correction.
- [x] Keep the consumer source unchanged between matched-provider passing runs and mixed-provider failing runs; assert computed values for native/wrapped f32/f64.
- [x] Declare 0.2.0 for the incompatible provider boundary and admit current Mnemosyne 0.10 without old pins or arithmetic changes.
- [x] Pass local 43-test native suite with zero skips, all configured native/docs/MSRV gates, the computed-value consumer and current supply-chain policy; preserve 74 registry lock records.
- [ ] Complete final-head fresh graph, source/normalized consumer, API, package and hosted acceptance with independent review.
- [ ] Obtain exact-head independent review and version/publication-scope acceptance; keep #100 review and all existing holds.

## H-ORDER-002 — Verify stage-time coefficients — current Atlas session

- [x] Reproduce the review finding: the autonomous `y' = y` fixture never
      reads the stage-time argument.
- [x] Add the generic non-autonomous `y' = t + y`, `y(0) = 1` refinement oracle
      with exact endpoint `2e - 2` for Euler, Midpoint, RK4, and
      Dormand–Prince.
- [x] Run the fixed-step Nextest binary (6/6), the locked all-featured Nextest
      suite (26/26), no-default-features Cargo check, and warning-denied
      all-target Clippy against the clean provider lane.
- [x] Run provider doctests (1/1) and warning-denied Rustdoc; both pass on the
      exact lane revision.
- [ ] Synchronize the exact hosted PR and Atlas pointer evidence.

## H-ORDER-001 — Verify observed order of accuracy by refinement — closed 2026-08-20

- [x] Add the closed-form `y' = y` refinement oracle for Euler, Midpoint, RK4,
      and Dormand–Prince, with dyadic refinement and discrete formal-order
      classification.
- [x] Cover the resolvable f32 non-embedded orders and retain the existing
      f32/f64 embedded execution tests where f32 fifth-order truncation is
      below the finite-step rounding floor.
- [x] Run exact-lane format, locked checks, warning-denied Clippy, Nextest,
      doctests, and Rustdoc; run the RK4 weight mutation control and restore
      the source.

## ATLAS-HORAE-AUDIT-073 — Isolated provider re-verification — closed 2026-08-16

- Owner: current Atlas session.
- Scope: `checklist.md` and `gap_audit.md`; no source or consumer changes.
- [x] Record locked isolated-provider gates, the umbrella-overlay lock
      boundary, and the remaining consumer/owner triggers with exact commands
      and evidence limits. Revision `1068651`; isolated format, Clippy,
      `cargo check`, 20/20 Nextest, 1 doctest, and rustdoc pass.

- [x] Land the required Aequitas/Eunomia quantity and scalar contracts for
      typed time/duration values.
- [x] Implement typed simulation time, explicit RK4 tableau, and fixed-step
      policy with failure-context preservation.
- [x] Implement adaptive step control with the generic f32/f64 adaptive-policy
      contract.
- [x] Implement `EventSchedule` with duplicate-skip and no-clip-without-
      crossing semantics.
- [x] Implement subcycle policy with bounded refinement and time alignment.
- [x] Pass formatting, both feature checks, warning-denied Clippy, Nextest
      (16/16 all-features), doctests, and the `--no-default-features` check.
- [x] Close the provider book (ATLAS-HORAE-PROVIDER-DOCS-001): adaptive,
      events, explicit systems, RK4, and subcycling chapters with runnable
      examples; mdBook build verified.
- [x] Record the f32 adaptive-policy contract in the book prose (PR #7).
- [x] Qualify event endpoint exactness with the Sterbenz condition, make the
      reported event instant authoritative, and test the large-magnitude case.
- [x] Replace the subcycle bit-identity claim with the derived ratio-three
      `gamma_4` reconstruction bound in code, tests, and book text.
- [x] Regenerate the ADR index from the existing canonical `Status: Accepted`
      header for ADR 0001.
- [x] Close H-004: make every book Rust example standalone, classify formula and
      diagram fences as text, and add the hermetic mdBook test to CI.
- [x] Remove all Atlas-overlay `[[patch.unused]]` records from the committed
      standalone lockfile; standalone offline locked metadata passes and the
      hosted locked book build passes in verify run `31859557127`.
- [x] Make the hosted book test invoke Cargo and rustdoc through the pinned
      toolchain; verify run `31859557127` passes the six-chapter book suite,
      and Pages book build `31859557311` passes.

## Open (consumer- or owner-gated)

- [x] Higher-order embedded adaptive tableaus: `DormandPrince` provides a
      seven-stage fifth/fourth-order pair and `step_embedded_into` returns the
      primary result plus a caller-owned local-error estimate.
- [x] Audit the stiff/implicit integration boundary against all current Atlas
      consumers. No production `ExplicitSystem` implementation or stepping
      call exists outside Horae tests/examples; Harmonia consumes typed time
      and subcycling, and Helios consumes `StepSize`. ADR 0001 remains the
      governing boundary, so no speculative implicit solver is added.
- [ ] Add implicit integration only when a second concrete residual/Jacobian
      consumer establishes the shared contract and bounded convergence oracle.
- [ ] Registry publication (`publish = false`; occupied name) — owner-gated.

## Evidence boundary

The lower-case `checklist.md` is the sole tracked provider checklist. Atlas
root `backlog.md` owns cross-repository hashes, hosted-run identifiers, and
registration history; this file records only the local completion state and
points inward to that SSOT to avoid duplicated evidence.

## HORAE-LOCKSTEP-074 — Aequitas/Eunomia consumer pin — closed 2026-08-16

- [x] Advance the locked Aequitas and Eunomia revisions to the fetched provider
      defaults without changing Horae's manifest dependency boundary.
- [x] Run the locked provider gates and verify both exact source revisions in
      `Cargo.lock`. Commit `65cb253` records Aequitas `5114cd12` and Eunomia
      `88c685f`; formatting, locked build/check, strict Clippy, 20/20 Nextest,
      1/1 doctest, rustdoc, seven-chapter mdBook tests, the example, and
      cargo-deny pass from outside the Atlas overlay.
- [x] Synchronize `backlog.md` and `gap_audit.md` with the local closure
      evidence.
- [x] Publish and merge the provider branch. PR #16 merged at `379f506`; hosted
      CI `31964298702` and Pages deployment `31964297946` pass.

## HORAE-LOCKSTEP-075 — Refresh merged Aequitas/Eunomia lock heads

- [x] Regenerate `Cargo.lock` outside the Atlas overlay to Aequitas
      `260ad10d` and Eunomia `85e590b7`; preserve the Git-sourced manifest
      boundary.
- [x] Run the local-graph format, locked metadata, all-feature and
      no-default-features checks, warning-denied Clippy, Nextest 20/20,
      doctest 1/1, rustdoc, and cargo-deny. Cargo-deny passes its four policy
      classes and reports only the existing unmatched-source warnings.
- [x] Record the evidence boundary: after restoring the committed standalone
      lock, root-overlay `cargo nextest --locked` refuses before test execution
      because local patches are not represented in the lock.
- [x] Commit and publish the lock-only provider increment as `9cc9fd8` on
      `codex/horae-lockstep-075`; open draft PR #19.
- [x] Collect the exact-head hosted `verify` and `supply-chain` checks, merge
      provider PR #19 at default `1ed6a17`, and record CI run `32202560133`
      plus Pages deployment `32202559349`. The Atlas parent may now consume
      this exact default head; parent overlay and exact-head gates remain
      owned by Atlas.

## gap-audit-2026-08-20 (owner: atlas-gap-audit)

Static, read-only scope-vs-delivery audit at detached head `a05dbeb` with a
clean tree. No build, test, lint, or Git state-changing command was run; every
claim below is a file-and-line or command-output citation. Findings are filed
as `H-*-0NN` items in `backlog.md`.

- [x] Orient: `git log --oneline -8`, `git status -sb` (`## HEAD (no branch)`),
      `git status --porcelain` (0 entries).
- [x] Read declared scope: `README.md`, `CHANGELOG.md` Unreleased,
      `docs/adr/README.md` and ADR 0001, `docs/book/SUMMARY.md`, `backlog.md`,
      `checklist.md`, `gap_audit.md`.
- [x] Measure the tree: one package, 1388 `src` LOC across 28 files (largest
      `src/integration/stepper.rs` at 243), 771 `tests` LOC, 71 `examples`
      LOC, no `benches/`, 23 `#[test]` functions plus one doctest.
- [x] Measure the conformance floor: zero `todo!`/`unimplemented!`, zero
      TODO/FIXME/HACK, zero `#[allow]`, one justified `#[expect]`
      (`examples/ordered_decay.rs:3`), zero production `unwrap()`, zero `dyn`
      in `src`, zero aliasing re-exports, zero files over 500 lines, zero
      type-suffixed identifiers, `lib.rs` and every `mod.rs` manifest-only.
- [x] Answer the integrator-coverage question: Euler, Midpoint, `Rk4`, and the
      `DormandPrince` 5(4) embedded pair exist; no leapfrog/Verlet, SSP-RK,
      implicit, or IMEX surface is declared or present, and ADR 0001 excludes
      implicit integration pending a second consumer contract.
- [x] Answer the order-verification question: no observed-order-of-accuracy
      refinement study exists for any method. Filed as H-ORDER-001.
- [x] Answer the stability/CFL question: `README.md` and ADR 0001 place
      stability and CFL criteria outside the boundary, so their absence is not
      a gap; two unsourced stability sentences in the book contradict that
      boundary and are filed as H-STAB-DOC-005.
- [x] Answer the adaptive-policy question: accept/reject, scale clamping,
      validation, and non-finite handling are real and tested against synthetic
      observations; the estimator-to-controller loop and step rejection retry
      are untested. Filed as H-ADAPT-LOOP-002. Event clipping at exact event
      times is genuinely tested (`tests/properties.rs:60`, `:87`,
      `tests/policy.rs:109`, `:161`).
- [x] Cross-check every README and Accepted-ADR claim against the code; two
      stale claims fixed in place, three deferred to backlog items.
- [x] Update `backlog.md` (11 DoR-shaped items), this section, and
      `gap_audit.md`.
- [ ] Not performed and not claimed: no gate was executed. `cargo fmt`,
      `clippy`, `nextest`, doctests, `cargo doc`, the example, `mdbook test`,
      and `cargo deny` were all out of scope for this pass; the most recent
      recorded green run is the hosted evidence under
      `ATLAS-HORAE-WORKFLOW-PIN-2026-08-20` in `backlog.md`, not this audit.
