<a id="h-implicit-nonfinite-20261020"></a>
## H-IMPLICIT-NONFINITE-20261020 — Reject false NaN convergence [patch] — open
- Owner: unclaimed; separate follow-up from the time-progress correction.
- Reproduced through the public API with finite `y' = -y`, `y0 = f64::MAX`, `h = 2`, tolerance `1e-8`: the solver returns `Ok` after four evaluations with output `[NaN]`; the exact backward-Euler target `MAX/3` is finite.
- Cause: the residual maximum ignores NaN because comparison with it is false; an all-NaN residual leaves norm zero. Overflowing finite-input intermediates reach this path without a system error.
- Acceptance: a successful solve certifies a finite residual within tolerance; report arithmetic failure or nonconvergence when nonfinite values arise. Cover mixed finite/NaN components, finite-input overflow, matrix/correction nonfiniteness and unchanged finite solutions. Public error/tolerance policy needs its own bounded implementation review. This issue remains unfixed.
