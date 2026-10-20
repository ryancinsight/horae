# Representable time progress — private correctness review

Source baseline: `0671c62cf710e7d70499f8593f6e2ae50e4e370a` (Horae main).
Current-dependency baseline: `8c107421673998f32819a85829601456928453fb`.
Atlas policy snapshot: `b553c42da72dcdeb2bb2644c60759dc465fb329a`.

## Contract and counterexample

For finite `t` and positive finite `h`, successful advancement requires
`fl(t + h) > t`. Positivity of `h` alone is insufficient. For a binary scalar
of precision `p`, `t = 2/epsilon = 2^p` has upward spacing two. Adding one is
the exact midpoint; round-to-nearest-even returns `t`. The original function
checks only finiteness and returns that unchanged time. Explicit stepping can
advance state while reporting `start == end`; a loop stopping on time then
has no progress guarantee. Event clipping also returns the stalled proposal.

The correction validates the represented endpoint without widening, tolerance
adjustments or changing step sizes. It reports `TimeError::NonAdvancingStep`. This new public variant is an additive
[minor] change, so the private candidate targets 0.3.0; the existing enum is
non-exhaustive.
Explicit and embedded stepping validate the end after dimension checks and
before evaluating the system, preparing stages or writing output. Existing
implicit stepping and event clipping already use the shared time advancement.

## Regression and controls

`tests/main/time_progress.rs` is added to the existing harness. The same leaf
runs on the dependency baseline and candidate. Baseline fails the advance,
integration and event regressions; two boundary controls pass. Candidate passes
all five tests. Tracing captures arrange/act/assert spans on failure.

- Native and wrapped `f32`/`f64` exercise the same generic assertions.
- Every fixed tableau, Dormand–Prince embedded stepping and backward Euler
  reject stalled endpoints before evaluations or output writes.
- Exact upward neighbors, negative times, odd-significand ties rounding up,
  zero crossing, subnormal progress and overflow retain their correct results.
- Dimension errors keep precedence; the existing 46 test assertions remain
  byte-identical. No timeout, feature, workflow or numerical coefficient changes.

Run the regression with:

```sh
cargo nextest run --locked --all-features -E 'test(time_progress::)'
```

## Local verification

The complete suite passes 51/51 with zero skips. Strict Clippy, warning-denied
Rustdoc, two doctests, all nine book chapters, no-default all-targets and MSRV
1.95 checks pass. API comparison passes 196 applicable checks (57 inapplicable).
`cargo package --locked` verifies the normalized package with registry Aequitas
0.3.1 and Eunomia 0.10. Supply-chain, committed lock, cast, artifact-budget and
exact-revision conformance controls pass. Existing duplicate Syn versions are
reported by cargo-deny without a policy failure. The conformance result does
not certify 38 cross-repository references whose provider stores are absent.

Independent review resolves the initial patch/minor classification finding by
requiring 0.3.0; production and regression bytes stay unchanged. The private
evidence receipt includes the final exact-head review and command records.

## Dependency and ownership boundary

Fresh compatible resolution retains Aequitas
`1bc917c4c909596b4fb16b4b6c239587f442c1a1` and Eunomia
`15fbb94c8f7ed8fa7bce4baf4167e2ffbedeceb3`, and advances Mnemosyne to
`0c789cd44e2abaece36e60a872db28625adefdf6`. No older dependency is used.
Test tracing uses registry packages already present in the baseline graph.

Open Horae PRs #96/#98/#99 concern providers. The H-ORDER-002 claim concerns
the existing stage-time oracle; neither it nor provider work is edited.
No `.agents`, repository `AGENTS.md` or repository skills exist in these fresh
source snapshots. The host's existing `AGENTS.md` is read without modification.

## Separate open findings

The private evidence includes executable public-API regressions for
`H-IMPLICIT-NONFINITE-20261020` and `H-IMPLICIT-LAST-20261020`. Both fail on the
current implementation and remain open. They are outside this minimal
time-progress correction; the backlog records exact analytical fixtures.

## Public-action hold

This deliverable is private and requires independent exact-head review.
No push, PR creation, merge, publication, tag, workflow dispatch or deployment
is performed or authorized by this review. Existing owner/service/release
holds remain unchanged.

The current main workflow can automatically publish and tag version changes.
Pages is enabled with legacy `main:/` source, in addition to the book workflow.
The `crates-io` environment has no protection rules. The five required main
checks are Lockfile integrity, MSRV (1.95), Workflows are valid, supply-chain
and verify; repository settings do not substitute for the user's mandatory
independent review. Recheck settings and publication scope before proposing
later public actions.
