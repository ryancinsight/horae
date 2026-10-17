# ADR 0003: Version the public quantity and scalar provider boundary

- Status: Proposed
- Date: 2026-10-17
- Class: `[major]`
- Relates to: [ADR 0001](0001-time-integration-boundary.md)

## Context

Horae accepts and returns Aequitas `Time<T>` through `StepSize` and `Instant`.
Its generic controllers, systems and integrators expose Eunomia `FloatElement`
and `RealField` bounds. These are public nominal types and traits.

[Provider PR #99](https://github.com/ryancinsight/horae/pull/99) moves Aequitas
from 0.2 to 0.3 and Eunomia from 0.9 to 0.10 without changing Horae source.
That does not make the migration compatible with existing generic consumers.
An unchanged external consumer runs with the old matched providers and with
#99's new matched providers. Pairing #99 with the old Aequitas identity fails
with E0308; using the old Eunomia bound fails with E0277. Internal tests alone
cannot establish compatibility across this boundary.

The same PR retains `mnemosyne-memory ^0.9.0` as a versioned Git dev-dependency.
A fresh root resolution fails because current Mnemosyne main declares 0.10.0;
the older committed lock masks this failure.

## Decision

Declare Horae 0.2.0 for the incompatible pre-1.0 public provider migration.
Retain #99's Aequitas 0.3/Eunomia 0.10 runtime requirements and advance the
Mnemosyne dev requirement to 0.10 against current approved defaults. Keep
source URLs, features, native arithmetic, integration algorithms and limits.
No compatibility shim, legacy provider pin or precision conversion is added.

`examples/provider_boundary.rs` is one generic unchanged consumer for the
quantity/scalar contract. Run it for native and wrapped f32/f64; assert exact
typed time advancement, duration, scaling and adaptive-report values. The
hierarchical native harness executes this same program.

## Acceptance and sequencing

This prerequisite is stacked on #99 and preserves its branch. It does not
include [invalid-order PR #100](https://github.com/ryancinsight/horae/pull/100),
whose current head and independent review remain separate. Once the provider
sequence is accepted, rebase that patch onto the new release line and renew
its exact-head validation and review. Its old-provider 0.1.1 API comparison
does not classify this provider boundary.

Fresh source and normalized-package resolution, unchanged external consumer
examples, API comparisons, all native/documentation/MSRV/supply-chain gates,
package verification and exact-head independent review precede acceptance.
A missing published Horae baseline is not a compatibility verdict. Main pushes
and schedules can plan publication; the current never-published name needs a
separately authorized first publish. Renew registry, ownership and publication
scope before merge. No merge, publication, credential change or service
deployment is authorized by this ADR or its implementation.
