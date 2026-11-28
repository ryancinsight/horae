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

## Final correction and acceptance

The sixteenth Newton correction was once applied without testing its iterate
against the tolerance, so a step that met the tolerance only there returned
`NonConvergence`. The loop now makes at most sixteen corrections and tests
the residual before each and once after the last: a step that exhausts the
budget costs one more residual evaluation, and the budget and the acceptance
rule are unchanged (horae#150). `tests/main/implicit_last_correction.rs`
asserts the output, residual evaluations and Jacobian evaluations of two
fixtures.

- `y' = -r·y` with the Jacobian reported as zero and `a = h·r = 1/2`: the
  Newton matrix is `I` and each correction multiplies the error by `-1/2`.
  For `y_n = 1` the residual after `k` corrections is `2^-(k+2)`. With
  `h = 2^-7` and `r = 64` every operation is exact in every shipped scalar,
  so the iterate is asserted bitwise. Tolerance `5e-6` or `2^-18` returns
  `Ok` after 18 evaluations and 16 Jacobians (before: `NonConvergence` after
  17); `2^-17` returns `Ok` after 17; `2^-19` returns `NonConvergence` after
  18. The board parameters `h = 0.01`, `r = 50` are asserted at `f64`.
- `y_n = 1/2`, `h = 1`, `f(y) = y - 1/2 - y²` with Jacobian `1 - 2y`, at
  `f64` and the wrapped `f64`. The residual is `g(y) = y²` and the Newton
  matrix is `2y`, so each correction halves the iterate from the predictor
  `1/4`: `y = 2^-(k+2)` and `g = 2^-(2k+4)` after `k` corrections, exactly in
  binary arithmetic. Tolerance `2^-34` returns `Ok` after 17 evaluations,
  `2^-36` returns `Ok` after 18 and `2^-37` returns `NonConvergence` after
  18. A 24-bit mantissa cannot hold `y²` down to `2^-36`, so `f32` is
  excluded.

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
