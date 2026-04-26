# GMvpn To Veil Audit

## Purpose

This note grounds the current Veil prototype against the existing GMvpn repository shape.

It is not a migration plan for a blind rewrite.
It is a responsibility map that helps Veil keep the useful control-plane behavior, isolate dataplane concerns, and avoid importing legacy structure wholesale.

## Repositories Audited

- `GMvpn/src/vpn_client/`
- `Veil/prototypes/veil-rust/`

## Current Reading

The current GMvpn repository is not only a VPN launcher.
It already contains several control-plane responsibilities:

- signed manifest verification and provider-profile compilation
- runtime contour assessment and platform capability checks
- session orchestration, probing, health checks, and recovery
- incident summarization and operator-facing diagnostics
- dataplane backend selection and backend process supervision

The current Veil Rust workspace is still much smaller, but its crate boundaries already match the intended long-term control-plane split:

- `veil-manifest`
- `veil-policy`
- `veil-routing`
- `veil-diagnostics`
- `veil-adapter-api`
- `veil-adapter-xray`
- `veil-core`
- `veil-cli`

That means Veil is directionally correct, but still far behind GMvpn in behavioral depth.

## Responsibility Map

### 1. Manifest, Trust, And Provider Profile Compilation

GMvpn facts:

- `src/vpn_client/security.py` verifies Ed25519 signatures for signed manifests.
- `src/vpn_client/provider_compiler.py` compiles logical server variants into endpoint-level runtime inputs.
- `src/vpn_client/models.py` carries `platform_capabilities`, schema versions, and provider-profile fields.

Veil target:

- primary home: `prototypes/veil-rust/veil-manifest`
- related policy contract: `prototypes/veil-rust/veil-policy`

Classification:

- keep as reference behavior: signature verification flow and provider-profile compilation rules
- rewrite later: typed manifest schema, signature handling, and logical-server compilation in Rust
- do not copy directly: Python manifest model as the long-term Veil source of truth

Reasoning:

This area is core control-plane value and belongs inside Veil, but it should be re-expressed as typed Rust domain models instead of porting the Python surface mechanically.

### 2. Policy Resolution And Runtime Contract Rules

GMvpn facts:

- `src/vpn_client/policy.py` validates structured policy blocks for health, runtime support, and transport failure handling.
- `src/vpn_client/runtime_support.py` classifies runtime contours such as `mvp-supported`, `planned`, and `bridge-only`.

Veil target:

- primary home: `prototypes/veil-rust/veil-policy`
- supporting manifest input: `prototypes/veil-rust/veil-manifest`

Classification:

- keep as reference behavior: runtime support assessment and structured policy validation
- rewrite later: typed policy engine in Rust
- preserve conceptually: explicit support-tier language and contract-mismatch reporting

Reasoning:

This is squarely Veil control-plane logic. The current Rust policy slice is still much thinner than GMvpn and should evolve toward contract-aware policy, not just backend allowlists.

### 3. Routing, Scheduling, Cooldown, And Failover

GMvpn facts:

- `src/vpn_client/session.py` coordinates endpoint scheduling, probing, connect attempts, and retry behavior.
- `src/vpn_client/session.py` depends on endpoint ordering and cooldown-related state through scheduler and state layers.
- `src/vpn_client/incident.py` surfaces transport disablement, cooldowns, and recovery-triggered state.

Veil target:

- primary home: `prototypes/veil-rust/veil-routing`
- orchestration integration: `prototypes/veil-rust/veil-core`
- report contract: `prototypes/veil-rust/veil-dry-run`

Classification:

- keep as reference behavior: endpoint scheduling, cooldown, and incident-aware fallback logic
- rewrite later: routing and failover engine in Rust
- current gap: Veil routing only selects between backend candidates, not endpoints or recovery states

Reasoning:

This is one of the biggest behavior gaps between GMvpn and Veil today.
Veil has the right crate name, but not yet the route-selection depth that GMvpn already exercises.

### 4. Dataplane Backend Boundary

GMvpn facts:

- `src/vpn_client/dataplane.py` defines a backend interface, backend routing, and process supervision.
- `src/vpn_client/cli.py` can select `null`, `linux-userspace`, `xray-core`, `ios-bridge`, and `routed`.
- backend choice is explicit and partly validated against platform support.

Veil target:

- primary home: `prototypes/veil-rust/veil-adapter-api`
- concrete backend adapters: `prototypes/veil-rust/veil-adapter-xray` and `prototypes/veil-rust/veil-adapter-mock`
- orchestration integration: `prototypes/veil-rust/veil-core`

Classification:

- keep as reference behavior: backend capability checks and backend selection boundaries
- wrap as adapter concept: Xray-specific execution details
- replace later: GMvpn monolithic dataplane module should become smaller adapter implementations behind stable contracts

Reasoning:

GMvpn already proves the value of a backend boundary, but the Python module still mixes contract, router, process supervision, and runtime details.
Veil should preserve the boundary while separating those concerns more cleanly.

### 5. Platform Adapters And Network Stack Application

GMvpn facts:

- `src/vpn_client/platform_adapters.py` cleanly distinguishes Linux from placeholder platform adapters.
- `src/vpn_client/cli.py` treats network-stack choice separately from client platform and dataplane selection.

Veil target:

- no first-class Rust crate yet
- likely future homes: `veil-core` plus a future explicit platform adapter crate or submodule

Classification:

- keep as reference behavior: separation of network adapter from dataplane backend
- use as reference architecture: client-platform versus local adapter distinction
- rewrite later: platform adapter contract in Rust when Veil moves beyond dry-run-only orchestration

Reasoning:

This area is important, but it should not be forced prematurely into the current small Rust prototype.
The concept is worth preserving even before a concrete Rust implementation exists.

### 6. Diagnostics, Incident Summary, And Operator Guidance

GMvpn facts:

- `src/vpn_client/incident.py` produces operator-facing incident summaries from runtime state and failure context.
- `src/vpn_client/session.py` returns structured outcome details such as selected endpoint, transport, failure class, and reason code.

Veil target:

- primary home: `prototypes/veil-rust/veil-diagnostics`
- shared report contract: `prototypes/veil-rust/veil-dry-run`

Classification:

- keep as reference behavior: diagnostics built from typed reasons rather than opaque logs
- rewrite later: redacted operator-facing diagnostics in Rust
- current gap: Veil diagnostics are still dry-run snapshots, not incident-oriented operational summaries

Reasoning:

GMvpn is already much closer to the intended Veil support story here than the current Rust prototype.
This is a good next-layer target because it strengthens control-plane observability without forcing real network execution.

## Classification Summary

Keep as reference behavior:

- signed manifest verification
- provider-profile compilation
- runtime support assessment
- endpoint scheduling and cooldown logic
- incident-oriented diagnostics
- separation of platform adapter from dataplane backend

Wrap as adapter concept:

- Xray-specific backend execution and typed config generation
- future backend-specific process management

Use as reference architecture:

- client-platform versus local runtime adapter split
- support-tier language such as `mvp-supported`, `planned`, and `bridge-only`

Rewrite later in Rust:

- manifest schema and trust pipeline
- richer policy engine
- endpoint routing and failover
- diagnostics and incident summaries
- platform adapter contract

Keep out of Veil control plane as a direct import:

- backend-specific monolithic runtime orchestration inside one module
- Python data models as the long-term public contract

## Current Risks

- Veil routing currently models backend eligibility, while GMvpn already models endpoint-level scheduling and transport-aware recovery. That is the largest architectural gap.
- Veil manifest and policy slices are still demo-shaped and do not yet carry provider-profile richness, runtime support tiers, or platform capability contracts.
- Veil diagnostics already have stable reason codes, which is good, but they do not yet explain runtime incidents with the same depth as GMvpn.
- A direct port of `GMvpn/src/vpn_client/dataplane.py` into Veil would likely recreate the same mixing of contracts, routing, supervision, and backend details that Veil is trying to separate.

## Recommended Next Slice

The next small reviewable slice should deepen Veil in one place where GMvpn already demonstrates real control-plane value without forcing production runtime changes.

Preferred next slice:

- add typed runtime support assessment and support-tier reasoning to `veil-manifest`, `veil-policy`, and `veil-dry-run`

Why this slice:

- it is repository-grounded in existing GMvpn behavior
- it strengthens Veil as a control plane
- it avoids a premature jump into full endpoint scheduler or platform runtime work
- it improves diagnostics and policy explainability with relatively low blast radius
