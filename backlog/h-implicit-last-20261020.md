<a id="h-implicit-last-20261020"></a>
## H-IMPLICIT-LAST-20261020 — Check the final Newton correction [patch] — open
- Owner: unclaimed; separate follow-up from the time-progress correction.
- Reproduced with `f(y) = y - 1/2 - y²`, `df/dy = 1 - 2y`, `y0 = 1/2`, `h = 1`, tolerance `2^-36`. Predictor is `1/4`; each correction halves it. After correction 16, output is exactly `2^-18` and residual exactly `2^-36`, yet the solver returns `SolveError::NonConvergence`.
- Cause: residual checks precede the 16 corrections; the last result is never checked. Acceptance: check the final permitted iterate without exceeding the correction bound, retain rejection just below the residual tolerance, and preserve evaluation accounting. This issue remains unfixed.
