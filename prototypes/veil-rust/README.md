# Veil Rust Prototype Workspace

This workspace is the first Rust prototype slice for Veil.

Current purpose:

- establish reviewable crate boundaries from `docs/veil-architecture-rfc.md`
- provide dry-run-only scaffolding for the first adapter and control-plane surfaces

Current crates:

- `veil-core`
- `veil-adapter-api`
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

Validation commands once Rust toolchain is available:

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```
