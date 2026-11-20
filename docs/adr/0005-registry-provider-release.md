# ADR 0005: Publish the Aequitas 0.4 time boundary

- Status: Accepted
- Date: 2026-11-20
- Class: `[major]`

## Context

Atlas applications exchange public Rust types across independently published crates. Cargo gives nominal types different identities when their package source or incompatible version differs. A fresh registry lock must select one identity for each type shared across the boundary.

## Decision

Publish horae-time 0.3.0 from the verified provider source. Step sizes use Aequitas 0.4 quantities. The integration traits use Eunomia 0.12. Consumers upgrade the two providers together when they implement a system or construct a step size.

## Verification

Run package, API, and feature checks on the release revision. Verify the archived source and checksum and compile a consumer from an empty lockfile against the indexed versions.
