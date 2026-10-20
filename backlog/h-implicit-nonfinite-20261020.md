<a id="h-implicit-nonfinite-20261020"></a>
## H-IMPLICIT-NONFINITE-20261020 — Reject false NaN convergence [patch] — private review
- Owner: isolated NaN-convergence lane; target: 0.3.1, stacked on reviewed `bc52f7d7c42fdd76c5a1396b5dd2376ec1c05728`. The time-progress branch stays unchanged.
- Reproduced through the public API with finite `y' = -y`, `y0 = f64::MAX`, `h = 2`, tolerance `1e-8`: the solver returns `Ok` after four evaluations with output `[NaN]`; the exact backward-Euler target `MAX/3` is finite.
- Cause: the residual maximum ignores NaN because comparison with it is false; an all-NaN residual leaves norm zero. Overflowing finite-input intermediates reach this path without a system error.
- Correction: require every residual magnitude to be finite before reducing the maximum; return existing `SolveError::NonConvergence` on nonfinite divergence. No new public variant, tolerance-policy or Newton-budget change.
- Regression: four new tests fail on the baseline and pass with the correction for native/wrapped scalars: finite decay overflow, mixed finite/nonfinite components, overflowing exchange Jacobian/correction, and an exact finite solution after workspace reuse.
- Local native suite: 55/55; original 51 tests and time-progress leaf are preserved. Exact-head gate receipts accompany the private artifact; independent review and public acceptance remain held.
- `H-IMPLICIT-LAST-20261020` remains open; its final-iterate tolerance reproducer is retained separately. See `../docs/verification/implicit-nonfinite.md`.
