# Veil Rust Workspace

This directory is the first public Rust workspace skeleton for Veil.

Its purpose is to make the intended module boundaries visible on the public branch before the full prototype implementation is moved here in larger slices.

## Current Status

This workspace is intentionally small, but it is no longer only a static directory layout.

It now provides:

- a Cargo workspace root
- initial crate boundaries that match the public architecture docs
- minimal placeholder types and entrypoints
- an initial end-to-end dry-run path from `veil-cli` through `veil-core` into `veil-adapter-xray`

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
- `veil-manifest`
- `veil-policy`
- `veil-routing`
- `veil-diagnostics`
- `veil-cli`

## Current Dry-Run Shape

The current public dry-run slice is intentionally simple:

- `veil-adapter-api` defines a minimal `DryRunPlan`
- `veil-adapter-xray` returns a placeholder Xray dry-run plan
- `veil-core` turns that plan into a control-plane `DryRunReport`
- `veil-cli` prints the rendered report

This is meant to prove crate wiring and ownership boundaries before broader behavior lands.

## Validation

Once a Rust toolchain is available, validate the workspace with:

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

## Next Steps

The next reviewable slices should add typed manifest input, policy-aware dry-run context, and richer diagnostics without landing a broad rewrite in one jump.
