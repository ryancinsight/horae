# Finite residuals before implicit convergence

This correction is a [patch] change: it uses the existing non-exhaustive
`SolveError::NonConvergence` and adds no public API. The original draft
stacked it on a reviewed time-progress baseline; the takeover rebase carries
only the residual guard onto the current default branch, so the unpublished
time-progress work stays outside this change. The crate version stays at
0.2.2: the member reserves the next published version for its release
authority, and this change claims none.

## Contract and counterexamples

A successful Newton residual check requires every component of
`g = y - y_n - h*f(t_end, y)` to be finite and `max_i |g_i| <= tolerance`.
An ordered comparison with NaN is false, so reducing a maximum initialized at
zero can discard every NaN component and incorrectly certify zero residual.

For real decay `f(y)=-y`, finite `y_n=MAX`, `h=2` and tolerance `1e-8`, the
fused predictor is `-MAX`; subsequent residual arithmetic overflows. The
baseline eventually returns `Ok` after four evaluations with a NaN output,
although the exact backward-Euler solution `MAX/3` is finite. The correction
returns `NonConvergence` at the first nonfinite residual, after two evaluations.

A second finite-input fixture uses conservative exchange
`f(y)=k*[y_1-y_0, y_0-y_1]`, `k=MAX`, `h=2`, and the smallest subnormal initial
state in one component. The predictor and first residual are finite, but
`I-h*J` overflows, elimination forms `inf/inf` and the correction becomes NaN.
The next residual cannot certify convergence. The baseline instead reports
success after three evaluations; the correction reports `NonConvergence`.

## Minimal correction and preserved boundary

Check each residual magnitude for finiteness before its existing comparison
with the norm. Finite values retain the same operation order and result;
there is no widening, clamping, substituted equation, allocation or retry.
The dense solve, Newton count, system-error propagation, shape/time checks and
caller tolerance policy remain unchanged. Failed output remains an
unsuccessful trial state; no valid solution is certified by the error result.

The new regression leaf executes the same assertions for native and wrapped
f32/f64. It covers all-NaN and mixed finite/nonfinite components, finite-input
overflow, matrix/correction failure, and workspace reuse with the exact
finite vector solution `[4/(1+.5*2), 8/(1+.5*6)] = [2,2]`.
All four regressions fail on unchanged production source and pass after the
guard. On the current default branch the full suite passes 50/50 (46 existing
leaves plus the 4 new ones); the existing leaves, including the stage-time
oracle, remain byte-identical.

## Remaining defect and acceptance

`H-IMPLICIT-LAST-20261020` remains open. Its exact dyadic final-correction
fixture still yields output `2^-18`, residual `2^-36`, tolerance `2^-36`, and
`NonConvergence`; this guard does not change iteration placement or budget.
The separate public-API reproducer and its expected failure accompany the
private evidence receipt.

Exact baseline/candidate identities, regression hashes, native/lint/docs,
MSRV/API/package and preservation controls are recorded with the private
artifact.

## Takeover review and gates

The takeover reviewed the guard, the four regressions and the preserved
boundary on the current default branch and re-ran the member gates at the
review head: `cargo fmt --check`, `cargo clippy --all-targets --all-features
-- -D warnings`, `cargo nextest run --all-features` with 50 tests passing,
`cargo check --no-default-features`, `cargo test --doc --all-features` with
2 doctests passing, and `cargo doc --no-deps --all-features`. The four
regressions pass individually and the 46 pre-existing leaves are unchanged.

The review confirmed the guard is one finiteness check in the residual loop:
the checked arithmetic, operation order, dense solve, Newton budget, error
surface and caller tolerance policy are unchanged, and no allocation, retry or
widened accumulator is introduced. The test-only `tracing` and
`tracing-subscriber` dev-dependencies carry the arrange/act/assert spans; the
committed lock adds only those two entries.

This change claims no publication, tag, workflow dispatch, deployment,
credential, settings or permission change. Existing release and owner holds
remain in force, and the unpublished time-progress work keeps its own
review.
