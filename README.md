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
- the Xray adapter prototype now renders typed Xray-like config, exposes a dry-run preflight report, and keeps command building separate from lifecycle behavior
- the control-plane dry-run plan now includes backend preflight diagnostics, and the CLI can export a standalone redacted preflight artifact
- the adapter API now owns the generic dry-run preflight contract, so `veil-core` no longer hardcodes Xray-specific preflight wiring
- the prototype now includes a second `mock-backend` adapter so the shared adapter API path is exercised across more than one backend
- the adapter registry now exposes operator-facing capability metadata, and the CLI reports each backend's preflight, health, reload, typed-config, and dry-run-only flags
- support bundles now also include manifest-vs-registry compatibility diagnostics, so dry-run can show advertised backends, locally registered backends, and any mismatch between them
- the demo CLI now supports explicit `select-endpoint` and `select-backend` overrides, and those choices are carried into the dry-run plan rather than being handled only in the CLI layer
- the demo CLI can also temporarily disable a registered backend for mismatch testing, and the compact diagnostics summary now shows applied overrides directly in the top block
- the demo CLI now also has a `demo matrix` mode that runs a small built-in scenario set and prints concise per-scenario results
- the demo matrix can now run a single named scenario and export a structured JSON report for later inspection
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
