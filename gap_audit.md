# Horae ownership gap audit

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
`cargo metadata --locked --no-deps` passes. PR #14 must pass its hosted locked
build and book-test gate before this item is fully integrated.

## ATLAS-HORAE-TOOLCHAIN-071 — open 2026-08-14

The hosted run after `ATLAS-HORAE-LOCK-070` passed the locked build but failed
the book test with E0514: Cargo produced Horae, Aequitas, and Eunomia artifacts
with Rust 1.97 while mdBook invoked Rust 1.95 rustdoc. The CI book step now
derives the active override from `rustup show active-toolchain` and runs both
Cargo and mdBook through `rustup run`, keeping compilation and doctest tooling
on one pinned compiler. Hosted verification remains pending.

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
