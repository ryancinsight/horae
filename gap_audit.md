# Horae ownership gap audit

## HORAE-LOCKSTEP-075 — refreshed consumer lock 2026-08-18

The standalone Horae lock now follows the current merged Atlas provider heads:

- Aequitas: `260ad10dd5480eef8c82958d1d148199656db59e`
- Eunomia: `85e590b789505c66f5174043c2e7e851c20547a5`

Only `Cargo.lock` changed. From outside the Atlas overlay, format, locked
metadata, all-feature check, no-default-features check, warning-denied Clippy,
Nextest 20/20, doctest 1/1, and rustdoc pass. Cargo-deny passes advisories,
bans, licenses, and sources; its only output is the existing unmatched-source
warning for the two Git origins. The provider lock is ready for commit and
parent-gitlink integration. Commit `9cc9fd8` is pushed as draft PR #19; hosted
`verify` and `supply-chain` checks are queued, so merge and parent-gitlink
integration remain open.

## HORAE-LOCKSTEP-074 — Aequitas/Eunomia consumer pin — local closure 2026-08-16

Horae commit `65cb253` advances its standalone lockfile to the fetched
provider defaults without changing the manifest boundary:

- Aequitas: `5114cd12b6d7769628f2ebff83e26f9c5e19caed`
- Eunomia: `88c685f2a733df73f753f2351b2e4d2ede8ff478`

The provider branch passes `cargo fmt --all -- --check`, locked all-feature
build, locked no-default-features check, strict all-target Clippy, Nextest
(20/20), doctests (1/1), warning-denied rustdoc, the seven-chapter
`rustup run 1.97.0 mdbook test docs/book` suite, the `ordered_decay` example,
and cargo-deny advisories, bans, licenses, and sources. The book test was run
with the exact locked artifacts on the shared target after removing only the
verified Horae/Aequitas/Eunomia derived package outputs; no source or peer
checkout files were removed.

The Atlas parent overlay is intentionally excluded from this standalone
lockfile verification: running Cargo from `D:\atlas` injects local first-party
patches and requests overlay-only lock records. At the local-closure point,
hosted integration was pending; the merged-default evidence follows.

PR #16 merged at Horae default `379f506`. Post-merge CI run `31964298702`
passes the provider verification and supply-chain jobs, and Pages deployment
`31964297946` passes. The remaining cross-repository action is advancing the
parent Atlas Horae gitlink to this merged default.

## ATLAS-HORAE-AUDIT-073 — Isolated provider re-verification — closed 2026-08-16

The current provider default `1068651` was re-verified from outside the Atlas
umbrella directory so the parent `.cargo/config.toml` development overlay could
not alter the standalone lockfile. The exact locked commands were:

- `cargo fmt --check --manifest-path D:\atlas\worktrees\horae-audit-20260816\Cargo.toml`
- `cargo check --locked --all-features --all-targets --manifest-path D:\atlas\repos\horae\Cargo.toml`
- `cargo clippy --locked --all-targets --all-features --manifest-path D:\atlas\repos\horae\Cargo.toml -- -D warnings`
- `cargo nextest run --locked --all-features --manifest-path D:\atlas\repos\horae\Cargo.toml` — 20/20 passed
- `cargo test --locked --doc --all-features --manifest-path D:\atlas\repos\horae\Cargo.toml` — 1 passed
- `cargo doc --locked --no-deps --all-features --manifest-path D:\atlas\repos\horae\Cargo.toml`

The standalone checkout remains clean after verification. Running the same
locked commands from inside `D:\atlas` is a separate environment: the parent
overlay injects local first-party patches that are not represented in Horae's
standalone lockfile, so Cargo refuses the command before compilation and asks
to rewrite `Cargo.lock`. That is an umbrella verification/configuration
boundary, not evidence that the provider lockfile should be changed. The
provider lockfile remains the standalone and hosted source of truth.

These checks establish formatting, compilation, static diagnostics, test,
doctest, and documentation behavior only. They make no new runtime,
performance, memory, or hardware-backend claim. No source-level placeholder
markers were found, and the remaining H-001 through H-003 items below retain
their consumer- or owner-gated status.

## ATLAS-HORAE-EXACTNESS-069 — resolved 2026-08-14

The event contract now treats `EventClip::event()` as the authoritative
boundary and states the Sterbenz precondition required for exact reconstruction
from `start + step`. A regression at `start = 1e8` and a `1e-6` event offset
passes under that precondition. The subcycle contract no longer claims
bit-identical parent reconstruction for every ratio; the ratio-three `f64`
policy documents and tests the derived `gamma_4` bound for its four rounded
operations. The root Atlas backlog owns the cross-repository integration
record.

The generated ADR index now records ADR 0001's existing `Accepted` status;
the index is derived state and no decision content changed.

## Boundary

Horae owns typed simulation time, explicit tableaus, adaptive control, event
scheduling, and subcycle policy. It does not own equation formulation,
stiff/implicit stepping, or consumer solver policy.

## Provider verification (2026-08-12)

- Strict all-target check with `-D warnings`: pass.
- Clippy `-D warnings` (all-targets/all-features): pass.
- Nextest (all-features): 16/16 pass, 0 skipped.
- Doctests: pass.
- Hermetic mdBook Rust examples: pass for all six chapters in a staged
  dependency view; the clean-checkout CI gate is now committed.
- `--no-default-features` check: pass.
- Local planning trail (backlog/checklist/gap_audit) authored at this audit.

## ATLAS-HORAE-LOCK-070 — resolved locally 2026-08-14

The committed `Cargo.lock` contained 55 `[[patch.unused]]` records emitted by
the Atlas umbrella overlay. Those records are not valid standalone lock state:
the hosted `cargo build --all-features --locked` gate rejected the lock before
book tests ran. The lock now has zero overlay records and records the Git
sources for Aequitas and Eunomia directly. Standalone offline
`cargo metadata --locked --no-deps` passes. Hosted verify run `31859557127`
passes the locked build and six-chapter book suite; Pages book build
`31859557311` also passes.

## ATLAS-HORAE-TOOLCHAIN-071 — resolved 2026-08-14

The hosted run after `ATLAS-HORAE-LOCK-070` passed the locked build but failed
the book test with E0514: Cargo produced Horae, Aequitas, and Eunomia artifacts
with Rust 1.97 while mdBook invoked Rust 1.95 rustdoc. The CI book step now
derives the active override from `rustup show active-toolchain` and runs both
Cargo and mdBook through `rustup run`, keeping compilation and doctest tooling
on one pinned compiler. Verify run `31859557127` passes the full workflow,
including book tests, doctests, rustdoc, and the example.

## Gap inventory

| ID | Description | Status |
|----|-------------|--------|
| H-001 | Higher-order embedded adaptive tableaus (Dormand-Prince family) | Open — consumer-gated; current RK4/adaptive surface satisfies integrators |
| H-002 | Stiff/implicit integration policy | Open — explicit-policy-only boundary today |
| H-003 | Registry publication (`publish = false`, occupied name) | Open — owner-gated facade/publication follow-up |

No delivered source-level gaps remain: no `TODO`/`FIXME`/`unimplemented!`
markers exist in `src/`, and the Rust gates are green at the audited revision.
H-004 is closed: Rust examples are standalone through hidden setup, prose
formulas and diagrams use non-Rust fences, and CI builds the locked dependency
set before running the full book test.
