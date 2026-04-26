# Veil Rust Workspace

This directory is the first public Rust workspace skeleton for Veil.

Its purpose is to make the intended module boundaries visible on the public branch before the full prototype implementation is moved here in larger slices.

## Current Status

This workspace is intentionally small, but it is no longer only a static directory layout.

It now provides:

- a Cargo workspace root
- initial crate boundaries that match the public architecture docs
- minimal placeholder types and entrypoints
- an end-to-end dry-run path from `veil-cli` through `veil-core` into backend adapters
- typed manifest input and a first policy-aware decision summary in the dry-run report
- minimal backend selection through `veil-routing`
- adapter capability snapshots surfaced in both report and diagnostics output
- a redacted diagnostics view produced by `veil-diagnostics`
- a structured JSON-like diagnostics artifact for machine-readable inspection
- simple `policy-mismatch` and `mock-backend` CLI scenarios
- a first small unit-test layer for policy, routing, and backend selection behavior

It does not yet claim:

- a complete prototype
- production readiness
- real backend integration
- real manifest verification
- real routing or diagnostics behavior beyond the first dry-run report path

## Current Crates

- `veil-core`
- `veil-adapter-api`
- `veil-adapter-xray`
- `veil-adapter-mock`
- `veil-manifest`
- `veil-policy`
- `veil-routing`
- `veil-diagnostics`
- `veil-cli`

## Current Dry-Run Shape

The current public dry-run slice is intentionally simple:

- `veil-manifest` provides a typed demo manifest input
- `veil-policy` evaluates a minimal backend allow rule and a `policy-mismatch` scenario
- `veil-routing` selects a backend from available candidates when the CLI is not forcing one
- `veil-adapter-api` defines a minimal `DryRunPlan` and `AdapterCapabilities`
- `veil-adapter-xray` and `veil-adapter-mock` provide two backend choices with different capabilities
- `veil-core` turns those inputs into a control-plane `DryRunReport`
- `veil-diagnostics` builds a redacted support-facing view and a JSON-like artifact
- `veil-cli` prints the control-plane report, the redacted diagnostics block, and the structured artifact

This is meant to prove crate wiring and ownership boundaries before broader behavior lands.

## Validation

Once a Rust toolchain is available, validate the workspace with:

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

## Example CLI Runs

```bash
cargo run -p veil-cli
cargo run -p veil-cli -- policy-mismatch
cargo run -p veil-cli -- mock-backend
```

## Next Steps

The next reviewable slices should add richer manifest fields, explicit diagnostics structures, and route-scoring behavior without landing a broad rewrite in one jump.
