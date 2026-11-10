# Milestone ladder ([ADR 0004](../docs/adr/0004-release-scope-1-0.md))
<a id="M-coherence"></a> **M-coherence** (0.2.3) needs: (none) — provider and correctness coherence.
- exit: `H-PROVIDER-RANGE-20261019`, `H-ADAPT-ORDER-RANGE-001`, `H-PROVIDER-BOUNDARY-001`, and `H-ORDER-002` resolved with landed evidence on the default branch; `cargo fmt --all -- --check`, `cargo clippy --locked --all-targets --all-features -- -D warnings`, `cargo nextest run --locked --all-features`, `cargo test --locked --doc --all-features`, and `cargo doc --locked --no-deps --all-features` pass; no open correctness items on M-coherence.
<a id="M-stability"></a> **M-stability** (0.3.0) needs: M-coherence — stable contracts and consumer coherence.
- exit: integration contracts and examples are current; provider dependency pins/lock are coherent with fetched stack defaults for in-scope providers; full provider gates pass; no open correctness items.
<a id="M-1.0"></a> **M-1.0** (1.0.0) needs: M-stability — release-ready (authority-gated).
- exit: `cargo semver-checks` and package/release gates required for the declared release pass; README/book/rustdoc are current; no open correctness items.
