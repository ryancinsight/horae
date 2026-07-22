# Changelog

All notable changes to Horae are documented in this file.

## [Unreleased]

### Changed

- Align the direct Eunomia scalar dependency with Aequitas on the canonical
  `0.7.0` Git source, eliminating duplicate scalar type identities.

### Added

- Typed finite instants and positive step sizes over Aequitas time.
- Slice-based explicit-system evaluation with associated errors.
- Const-generic Euler, midpoint, and RK4 tableaus with reusable workspaces.
- Allocation-free fixed-step and adaptive reports.
- Borrowed event scheduling with exact step clipping.
- Zero-sized const-generic subcycle-ratio plans.
