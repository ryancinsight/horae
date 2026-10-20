<a id="h-time-progress-20261020"></a>
## H-TIME-PROGRESS-20261020 — Reject stalled represented time [minor] — private review
- Owner: isolated Atlas correctness lane; target: 0.3.0. Preserve the H-ORDER-002 oracle, provider PRs #96/#98/#99 and existing release holds.
- Baseline source: `0671c62cf710e7d70499f8593f6e2ae50e4e370a`; current-dependency baseline: `8c107421673998f32819a85829601456928453fb`.
- Reproduction: at `start = 2/epsilon`, `h = 1` rounds to the same instant. `advance`, Euler's `StepReport` and event clipping report success. The unchanged new regression leaf fails three tests on baseline and passes on the correction; precision fixtures derive from adjacent IEEE values.
- Correction: require `end > start` after finite addition; preserve dimension error precedence and preflight explicit/embedded endpoints before state work.
- Verification and public-action scope: see `../docs/verification/time-progress.md`. Final exact-head review and gate receipts accompany this private candidate. No public action is authorized.
