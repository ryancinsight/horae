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

## Aequitas 0.5 release

horae-time 0.4.0 replaces the public Aequitas 0.4 identity of 0.3.0
with the published Aequitas 0.5 identity. Pass the Aequitas 0.5 time quantities to StepSize and Instant.
Eunomia remains at 0.12.1. The release uses the reviewed source that already
selects Aequitas 0.5. The package version marks the breaking change.

Verify the normalized package with Rust 1.95, then build a registry consumer
which passes values of the shared types across the public boundary.
