# Current/future provider compatibility bridge [patch]

Target:0.2.2. Widen only Aequitas>=0.3,<0.5 and Eunomia>=0.10,<0.12; retain current lower bounds,
canonical Git sources,feature selections and every Rust implementation/test.
No adapter,compatibility alias,path patch,branch/rev pin,CI change or gate/limit
reduction is proposed. Athena's regenerated lock also replaces inherited
Moirai0.9 pins with current0.10 defaults. Current normal locks select merged
Aequitas0.3.1 and Eunomia0.10; unreleased providers are not selected in them.

Horae0.2.1's reviewed adaptive invalid-order repair and all46 native cases
remain unchanged. Its local Cargo publication approval/authentication hold is
preserved: this bridge does not retry that action, change credentials/settings,
or treat a source compatibility change as authorization for the held fallback.
Existing automatic main-publisher side effects need their normal final review.

## Evidence and regression controls

Three exact-baseline requirement probes reject Aequitas0.4 or Eunomia0.11 before
compilation. The widened exports pass real external typed-time/scalar-policy
consumers in four cases:current0.3.1/0.10,next-law0.4/0.10,next-scalar0.3.1/0.11,
and a probe-only combined0.4/0.11 range composition. Both value oracles execute
for primitive and wrapped single/double precision. Time advance,duration,
scaling,adaptive rejection and exact convergence thresholds are checked from
binary-exact inputs. These are computation and nominal-identity checks,not
successful-resolution assertions alone.

Future scalar source is independently reviewed Eunomia22daa6c67e3d0f16d346a312b6835ccbf351a708.
Future law source is independently reviewed Aequitas a6cb96fcc72fb312ac3d17341812d49dde015b28,
whose production is identical to cleared9a015229. The combined probe applies
only already-reviewed provider-range admission to that export; the final
canonical Aequitas bridge-first rebase still needs its own exact-head review.
Every path rewrite exists only in disposable probe exports outside this repo.
These probes do not establish publication,canonical final lock availability,
full backend future-cohort readiness or complete Python interoperability.

Current repository gates pass:46 native cases,zero skips;two doctests. Strict all-target/all-feature Clippy,
no-default targets,warning-denied Rustdoc,format and actual Rust1.95 all-target/
all-feature compilation pass. Unchanged supply-chain policy passes. Complete
actual locked baseline/head Rustdoc JSON passes223 explicit patch checks per
published package,with30 tool-defined skips; tools alone do not prove external
nominal dependency compatibility. No runtime compiler-floor bypass is used.

Athena's required accelerator tests initially fail to acquire Metal inside the
sandbox; the same unchanged suite passes with actual device access. No optional
device flag or disabled test is used to convert that failure to success.
Normal source-package verification and final hosted/independent review receipts
are recorded separately rather than inferred from local source tests.

## Coordination and rollout

The user approves the normal Eunomia0.11/Aequitas0.4/binding0.2/NumPy-PyO3.30
plan and ADR0020,bridge first,with mandatory tests/reviews. Eunomia owner01a13b11
retains Mnemosyne/Hermes bridges; coordinator01a14490 retains Leto/Hephaestus/
Apollo/Coeus. This change does not edit those branches. Full Athena future
backend coherence depends on their canonical bridge admissions; the isolated
core/time probes are not substituted for that final consumer gate.

Land backwards-compatible admissions on safe current providers before advancing
breaking provider defaults. Keep the combined NumPy0.30 change in still-unreleased
Eunomia0.11; if0.11 publishes first,a later breaking0.12 needs new admission beyond
this<0.12 ceiling. Rebase the law0.4/binding0.2 candidate after bridge140 and
regenerate/review its coherent final lock. No old-version pin or permanent
workaround replaces that rollout. This PR carries no merge/publication action.
Harmonia's separate numerical performance acceptance hold remains explicit.
