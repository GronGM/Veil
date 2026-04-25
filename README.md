# Veil

Veil is an adaptive VPN/proxy orchestration platform.

Current repository state:

- architecture RFC is documented in `docs/veil-architecture-rfc.md`
- the first Rust prototype workspace lives in `prototypes/veil-rust/`
- the workspace already includes a dedicated `veil-routing` crate for explainable route selection
- the workspace now also includes a dedicated `veil-diagnostics` crate for incident and support-facing output
- the prototype now includes a typed route policy layer with explicit allow/deny, known-good bias, cooldown handling, and predictable fallback
- the prototype now includes redacted manifest, route, and policy diagnostics views inside the support bundle
- the prototype CLI now defaults to a report-focused redacted output, with raw bundle JSON available behind an explicit flag
- the project is intentionally control-plane first
- dataplane backends are treated as replaceable adapters
- Linux plus Xray is the first honest MVP runtime contour inherited from the GMvpn reference work

What Veil is trying to become:

- a signed-manifest and provider-profile driven control plane
- an explainable routing and failover engine
- a diagnostics-first runtime with support bundles and observable decisions
- a Rust-priority modular platform for VPN/proxy orchestration

What Veil is not claiming today:

- full production readiness
- full cross-platform parity
- invisible or undetectable networking
- a blind rewrite of the original GMvpn codebase

Repository layout:

- `docs/` - architecture and project documents
- `prototypes/veil-rust/` - Rust prototype workspace

Validation note:

The current container does not include the Rust toolchain, so `cargo fmt`, `cargo check`, and `cargo test` still need to be run in a Rust-enabled environment.
