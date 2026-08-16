# Horae checklist

## ATLAS-HORAE-AUDIT-073 — Isolated provider re-verification — in progress

- Owner: current Atlas session.
- Scope: `checklist.md` and `gap_audit.md`; no source or consumer changes.
- Acceptance: record locked isolated-provider gates, the umbrella-overlay
      lock boundary, and the remaining consumer/owner triggers with exact
      commands and evidence limits.

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

- [ ] Higher-order embedded adaptive tableaus (Dormand-Prince family) when a
      consumer need emerges.
- [ ] Stiff/implicit integration policy.
- [ ] Registry publication (`publish = false`; occupied name) — owner-gated.

## Evidence boundary

The lower-case `checklist.md` is the sole tracked provider checklist. Atlas
root `backlog.md` owns cross-repository hashes, hosted-run identifiers, and
registration history; this file records only the local completion state and
points inward to that SSOT to avoid duplicated evidence.
