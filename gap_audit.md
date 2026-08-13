# Horae ownership gap audit

## Boundary

Horae owns typed simulation time, explicit tableaus, adaptive control, event
scheduling, and subcycle policy. It does not own equation formulation,
stiff/implicit stepping, or consumer solver policy.

## Provider verification (2026-08-12)

- Strict all-target check with `-D warnings`: pass.
- Clippy `-D warnings` (all-targets/all-features): pass.
- Nextest (all-features): 16/16 pass, 0 skipped.
- Doctests: pass.
- `--no-default-features` check: pass.
- Local planning trail (backlog/checklist/gap_audit) authored at this audit.

## Gap inventory

| ID | Description | Status |
|----|-------------|--------|
| H-001 | Higher-order embedded adaptive tableaus (Dormand-Prince family) | Open — consumer-gated; current RK4/adaptive surface satisfies integrators |
| H-002 | Stiff/implicit integration policy | Open — explicit-policy-only boundary today |
| H-003 | Registry publication (`publish = false`, occupied name) | Open — owner-gated facade/publication follow-up |

No source-level gaps remain in the delivered surface: no `TODO`/`FIXME`/
`unimplemented!` markers exist in `src/`, and all gates are green at the
audited revision.
