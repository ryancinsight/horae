# Changelog

All notable changes to Horae are documented in this file.

## [0.3.0] - Unreleased

### Fixed

- [minor] Add `TimeError::NonAdvancingStep` and reject positive steps whose rounded endpoint does not advance time,
  while preserving representable advances. Explicit and embedded stepping
  validate the endpoint before evaluating the system or writing output;
  implicit stepping and event clipping inherit the same time contract.

### Changed

- Refresh Mnemosyne to the current compatible default and add test-only
  tracing using already-resolved registry packages. Existing provider ranges,
  test assertions, timeout limits and workflows remain unchanged.
- Record two independently reproduced, unfixed implicit stopping defects:
  nonfinite residuals can report convergence, and the final permitted Newton
  correction is not checked. Neither is fixed by the time-progress patch.

### Acceptance hold

This private review candidate does not authorize a push, merge, publication,
tag, workflow dispatch or deployment. Existing provider and publication holds
remain in force; exact-head independent review is required before public work.

## [0.2.2] - Unreleased

### Changed

- [patch] Admit Aequitas>=0.3,<0.5 and Eunomia>=0.10,<0.12,preserving current-provider support and all Rust
  implementation/test/feature bytes. Current canonical lock and real typed
  current/future consumer probes verify the bridge; final provider ordering,
  exact-head review and hosted/publication gates remain required.

## [0.2.1] - Unreleased

### Fixed

- Return `AdaptiveError::InvalidOrder` for unsupported const orders including
  `usize::MAX`, avoiding associated-constant overflow during compilation.
  Preserve the existing `1..u32::MAX` supported range, valid-order results,
  and nonfinite-observation precedence; correct the error text's upper bound.

### Acceptance hold

The reviewed 0.2.0 provider tree integrates through PR #101 at
[accepted main](https://github.com/ryancinsight/horae/commit/ab42e5707a3b9a3cc080cfbbb8aa8e36bd4fc56f); first publication is still pending.
This separate next-patch draft must not merge or publish until the exact reviewed
0.2.0 source is accepted on the registry and this changed head receives renewed
checks and independent review. No registry upload, tag or credentials change
forms part of this rebase.

## [0.2.0] - Unreleased

### Breaking

- Version the public provider boundary as Horae 0.2.0: `Time<T>` uses
  Aequitas 0.3 and generic scalar bounds use Eunomia 0.10. Existing consumers
  must align their provider identities. ADR 0003 records the externally
  reproduced quantity-type and generic-bound failures; arithmetic is unchanged.

### Fixed

- Admit current Mnemosyne 0.10 in the versioned dev dependency so fresh
  repository resolution no longer depends on an obsolete provider lock.
  Retain the runtime provider updates from #99 and advance required transitive
  provider pins to their current defaults without changing registry records.


### Changed

- Refresh the Pages caller to the current Atlas reusable workflow revision;
  retain the existing package-staged executable book gate. Exact hosted CI,
  mdBook, and Pages deployment runs pass at merged default `c2e7766`; live
  Pages returns HTTP 200.

- Qualify event endpoint reconstruction with the Sterbenz exact-subtraction
  condition and document `EventClip::event()` as the authoritative boundary;
  add a large-magnitude regression case.
- Replace the unconditional subcycle parent/child bit-identity claim with the
  derived `gamma_4` rounding bound for the ratio-three `f64` contract.
- Document and pin the `EventSchedule` boundary semantics: `next_after`
  skips events equal to the start instant (duplicate notifications cannot
  cause a zero-length step or re-trigger), and `clip_step` reports an event at
  or before the proposed endpoint — otherwise the original step is returned
  unchanged with no event. A value-semantic test
  covers the duplicate-skips, no-clip-without-crossing, and empty-schedule
  cases.
- Align the direct Eunomia scalar dependency with Aequitas on the canonical
  `0.8.0` Git source, eliminating duplicate scalar type identities.
- Force scalar `Mul<T>` dispatch in `StepSize::scaled` to disambiguate the
  `Quantity × Quantity` and `Quantity × T` impl paths; the multiplicative impl
  captured inference when both eunomia surfaces were in scope, producing a
  `Quantity<T, _>` mismatch at `Self::new` (`step_size.rs:54`).

### Added

- Dormand--Prince fifth/fourth-order embedded stepping with a caller-owned
  local-error estimate and seven-stage workspace reuse.
- A generic adaptive-policy contract test for `f32`, covering mixed-tolerance
  acceptance/rejection and non-finite observations alongside the existing
  `f64` boundary suite; this pins both supported real scalar monomorphizations.
- Typed finite instants and positive step sizes over Aequitas time.
- Slice-based explicit-system evaluation with associated errors.
- Const-generic Euler, midpoint, and RK4 tableaus with reusable workspaces.
- Allocation-free fixed-step and adaptive reports.
- Borrowed event scheduling with authoritative event-boundary clipping.
- Zero-sized const-generic subcycle-ratio plans.
