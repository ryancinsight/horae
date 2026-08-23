# Horae checklist

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
