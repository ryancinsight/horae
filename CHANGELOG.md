# Changelog

All notable changes to Horae are documented in this file.

## [Unreleased]

### Changed

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

- A generic adaptive-policy contract test for `f32`, covering mixed-tolerance
  acceptance/rejection and non-finite observations alongside the existing
  `f64` boundary suite; this pins both supported real scalar monomorphizations.
- Typed finite instants and positive step sizes over Aequitas time.
- Slice-based explicit-system evaluation with associated errors.
- Const-generic Euler, midpoint, and RK4 tableaus with reusable workspaces.
- Allocation-free fixed-step and adaptive reports.
- Borrowed event scheduling with authoritative event-boundary clipping.
- Zero-sized const-generic subcycle-ratio plans.
