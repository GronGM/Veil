# Veil Rust Prototype Workspace

This workspace is the first Rust prototype slice for Veil.

Current purpose:

- establish reviewable crate boundaries from `docs/veil-architecture-rfc.md`
- provide dry-run-only scaffolding for the first adapter and control-plane surfaces
- exercise a typed policy layer before real networking and process supervision exist

Current crates:

- `veil-core`
- `veil-adapter-api`
- `veil-adapter-mock`
- `veil-adapter-xray`
- `veil-diagnostics`
- `veil-manifest`
- `veil-policy`
- `veil-routing`
- `veil-cli`

Intentional limitations:

- no real networking or privileged operations
- no real Xray binary integration
- no real signature verification yet
- no dynamic plugin loading
- no production readiness claims

Recent prototype slice:

- `veil-policy` now carries typed route policy structs for backend and transport allow/deny, region preferences, cooldown gating, known-good bias, and fallback mode
- `veil-routing` now uses those policy decisions to reject ineligible routes with explainable reasons
- `veil-cli demo` now accepts simple policy overrides and prints rejected routes separately from the selected route summary
- `veil-cli demo --policy-file <path>` now accepts partial JSON policy overrides, then lets CLI flags override those file values
- `support_bundle` now includes a dedicated `route_diagnostics` section with selected route, fallback state, and rejected candidate reasons
- `support_bundle` now also includes `redacted_route_diagnostics`, which masks token-like values, private keys, credential-bearing URLs, and UUID-like secrets by default
- `support_bundle` now includes `redacted_policy_diagnostics`, which preserves policy shape while masking secret-like override values in lists and strings
- `veil-cli demo` now prints a compact human-readable diagnostics summary before the detailed JSON sections
- `support_bundle` now includes `redacted_manifest_diagnostics`, so the safe diagnostics surface covers manifest, route, and policy together
- `veil-cli demo` now defaults to report-focused redacted output and prints raw route/bundle JSON only when `--raw-json` is explicitly requested
- `veil-cli demo --export-redacted-bundle <path>` now writes a redacted diagnostics artifact to disk for later review or attachment
- `veil-adapter-xray` now renders a typed Xray-like config model, builds a separate command spec, and reports dry-run preflight details without requiring a real `xray` binary
- `veil-core` now carries backend preflight details into the dry-run plan, and `veil-cli demo --export-redacted-preflight <path>` can write a standalone safe artifact for review
- `veil-adapter-api` now defines the generic dry-run preflight contract, so future adapters can plug into the same control-plane path without backend-specific branching in `veil-core`
- `veil-adapter-mock` now provides a second dry-run backend so route selection and adapter preflight can be exercised across multiple registered backends
- adapter registry snapshots now include operator-facing capabilities so CLI output can show which backends support preflight, health checks, reload, typed config rendering, and dry-run-only execution
- support bundles now include adapter compatibility diagnostics that compare manifest-advertised backends with the locally registered adapter set
- `veil-cli demo` now supports explicit backend and endpoint selection overrides so dry-run can reproduce `xray-core`, `mock-backend`, or mismatch paths on demand

Example policy files:

- `examples/policies/deny-xray.json` - forces a policy-level backend rejection for the demo path
- `examples/policies/prefer-eu-stable.json` - biases selection toward European routes with a stronger stability posture
- `examples/policies/strict-fallback.json` - disables known-good fallback so cooldown and retry rules stay strict

Example demo commands once Rust toolchain is available:

```bash
cargo run -p veil-cli -- demo --policy-file examples/policies/deny-xray.json
cargo run -p veil-cli -- demo --policy-file examples/policies/prefer-eu-stable.json
cargo run -p veil-cli -- demo --policy-file examples/policies/strict-fallback.json --deny-transport https
cargo run -p veil-cli -- demo --policy-file examples/policies/prefer-eu-stable.json --raw-json
cargo run -p veil-cli -- demo --policy-file examples/policies/prefer-eu-stable.json --export-redacted-bundle output/demo-redacted-bundle.json
cargo run -p veil-cli -- demo --export-redacted-preflight output/demo-redacted-preflight.json
cargo run -p veil-cli -- demo --select-backend mock-backend
cargo run -p veil-cli -- demo --select-endpoint edge-1
```

Validation commands once Rust toolchain is available:

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```
