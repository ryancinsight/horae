# ADR 0006: Own civil dates and wall-clock timestamps in Horae

- Status: Proposed
- Class: [minor] [arch]
- Scope: horae-civil-time 0.1.0 and its first-party consumers

## Decision

Give Gregorian rules and UTC report timestamps a separate package in Horae.
Consumers can use the core date and timestamp types on their own, with optional
clock access and optional Serde support. Keep DICOM value formats, repair, and
partial precision in RITK. Domain reports choose their labels and capture time
at their boundaries.

## Alternatives

Sharing date arithmetic through ritk-dicom makes unrelated reports depend on a
medical format package. Duplicating calendar arithmetic across callers makes
leap-day and pre-epoch behavior drift. A separate package keeps the common
arithmetic dependency small and the simulation-time API focused.

## Representation and verification

Dates cover Gregorian years 0000 through 9999. Timestamps hold a signed Unix
second and an exact nanosecond fraction. Leap-second encodings retain their
position between consecutive Unix seconds. The system-clock adapter rejects a
conversion that loses that distinction. Calendar inversion uses the unique
interval between consecutive year starts. Contract tests cover a complete
400-year cycle, the Unix epoch, offsets crossing midnight, fractions, and
existing RFC 3339 JSON records.
