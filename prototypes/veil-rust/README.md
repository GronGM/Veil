# Veil Rust Workspace

This directory is the first public Rust workspace skeleton for Veil.

Its purpose is to make the intended module boundaries visible on the public branch before the full prototype implementation is moved here in larger slices.

## Current Status

This workspace is intentionally minimal.

It provides:

- a Cargo workspace root
- initial crate boundaries that match the public architecture docs
- minimal placeholder types and entrypoints

It does not yet claim:

- a complete prototype
- production readiness
- real backend integration
- real manifest verification
- real routing or diagnostics behavior

## Current Crates

- `veil-core`
- `veil-adapter-api`
- `veil-adapter-xray`
- `veil-manifest`
- `veil-policy`
- `veil-routing`
- `veil-diagnostics`
- `veil-cli`

## Validation

Once a Rust toolchain is available, validate the workspace with:

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

## Next Steps

The next reviewable slices should wire these crates together gradually instead of landing a broad rewrite in one jump.
