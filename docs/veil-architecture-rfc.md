# Veil Architecture RFC

## Overview

This document defines the initial architecture for Veil as an adaptive VPN/proxy orchestration platform evolved from GMvpn without a blind rewrite.

Core architectural decision:

- Veil is control-plane first
- dataplane backends are replaceable adapters
- transport behavior is replaceable adapter logic
- runtime configs are generated artifacts, not the main user-facing config surface

## Goals and Non-Goals

### Goals

- preserve the tested control-plane semantics already proven in GMvpn
- separate control plane from dataplane more cleanly than the current single-package Python layout in the reference repository
- define a Rust-first module map for Veil MVP
- keep Xray as one backend adapter rather than the architecture itself
- make routing, failover, health, and policy decisions inspectable and explainable
- keep Linux + Xray as the first honest MVP runtime contour

### Non-Goals

- full rewrite of GMvpn in one step
- dynamic plugin loading in MVP
- claiming cross-platform parity before real adapters exist
- making users hand-edit backend JSON as the primary workflow
- making unsupported claims about invisibility, censorship resistance, or indistinguishability

## Module Map

The smallest useful Veil MVP module map is:

- `veil-core`: session orchestration, lifecycle coordination, runtime supervision
- `veil-manifest`: signed manifest loading, validation, compatibility, provider-profile handling
- `veil-policy`: declarative policy resolution and incident guidance
- `veil-routing`: endpoint scoring, cooldown, failover, known-good reuse, retry budgeting
- `veil-diagnostics`: incident summary, support bundle shaping, redaction rules
- `veil-adapter-api`: in-process adapter traits and contracts
- `veil-adapter-xray`: first concrete dataplane adapter
- `veil-cli`: local operator shell and dry-run entrypoint

Reference mapping from GMvpn:

- `config.py` and `provider_compiler.py` responsibilities become `veil-manifest`
- `session.py` becomes `veil-core`
- `scheduler.py` and `state.py` become `veil-routing`
- `policy.py` becomes `veil-policy`
- `incident.py` and support-bundle shaping become `veil-diagnostics`
- `dataplane.py` becomes `veil-adapter-api`
- `xray.py` becomes `veil-adapter-xray`

## Architecture Diagram

```text
                    +----------------------+
                    |      veil-cli        |
                    | local UX / dry-run   |
                    +----------+-----------+
                               |
                               v
                    +----------------------+
                    |      veil-core       |
                    | session orchestration|
                    | runtime supervision  |
                    +---+------+-----+-----+
                        |      |     |
            +-----------+      |     +-------------+
            |                  |                   |
            v                  v                   v
   +----------------+  +----------------+  +-------------------+
   | veil-manifest  |  | veil-policy    |  | veil-diagnostics  |
   | signed inputs  |  | policy resolve |  | bundles, incident |
   | schema compat  |  | guidance       |  | summaries         |
   +--------+-------+  +--------+-------+  +---------+---------+
            |                   |                      ^
            v                   v                      |
                +-------------------------------+      |
                |         veil-routing          |------+
                | score, cooldown, failover    |
                | known-good, retry budget     |
                +---------------+--------------+
                                |
                                v
                    +-------------------------+
                    |    veil-adapter-api     |
                    | backend/transport traits|
                    +-----------+-------------+
                                |
                 +--------------+--------------+
                 |                             |
                 v                             v
      +-----------------------+     +-----------------------+
      |  veil-adapter-xray    |     | veil-adapter-mock     |
      | render/apply/check    |     | demo/test backend     |
      +-----------------------+     +-----------------------+
```

## Module Design

### `veil-core`

- Responsibility: own session orchestration, connect/reconnect/failover lifecycle, runtime tick coordination, and startup recovery.
- Inputs: normalized manifest/profile objects, resolved policy objects, ordered route candidates, adapter registrations, persisted runtime state.
- Outputs: session reports, selected route context, lifecycle events, runtime supervision state.
- Public Interfaces: `SessionEngine`, `RuntimeSupervisor`, `StartupRecoveryCoordinator`.
- Dependencies: `veil-manifest`, `veil-policy`, `veil-routing`, `veil-diagnostics`, `veil-adapter-api`.
- Hot-swappable Parts: no direct backend logic; all dataplane work goes through adapter traits.
- MVP Scope: connect, failover, stale runtime recovery, health-driven degradation, Linux-first runtime supervision.
- v1 Scope: broader platform runtime coordination and richer maintenance loops.
- v2/Research Scope: distributed control-plane coordination or remote policy push channels.
- Failure Modes: lost runtime marker, invalid adapter registration, inconsistent route state, restart loops.
- Test Strategy: deterministic unit tests for session states and recovery paths.

### `veil-manifest`

- Responsibility: verify signatures, enforce schema compatibility, validate provider-profile inputs, normalize high-level user/provider config into internal types.
- Inputs: signed manifest bytes, public keys or trust anchors, optional last-known-good cache, provider compiler inputs.
- Outputs: normalized `ProviderManifest`, `ProviderProfile`, compatibility diagnostics, cached validated snapshot.
- Public Interfaces: `ManifestLoader`, `ManifestVerifier`, `ProviderProfileCompiler`, `CompatibilityChecker`.
- Dependencies: crypto libraries, serde models, optional diagnostics hooks.
- Hot-swappable Parts: trust root source and manifest transport can vary; schema rules remain core-owned.
- MVP Scope: signed manifests, schema version checks, provider-profile compilation, platform capability validation.
- v1 Scope: key rotation policy, richer profile families, migration helpers.
- v2/Research Scope: federated provider distribution or remote trust policy management.
- Failure Modes: signature mismatch, version mismatch, unsupported adapter section, duplicate IDs, expired manifest.
- Test Strategy: compatibility and signature tests plus compiler tests.

### `veil-policy`

- Responsibility: resolve network policy, runtime support policy, session health policy, transport failure policy, and incident guidance overrides.
- Inputs: normalized manifest/profile models, local operator overrides, platform context, route outcome context.
- Outputs: resolved policy structs and human-readable guidance objects.
- Public Interfaces: `PolicyResolver`, `IncidentGuidanceResolver`, `RuntimeSupportResolver`.
- Dependencies: `veil-manifest`, shared model types.
- Hot-swappable Parts: policy data is replaceable; evaluation engine stays core-owned.
- MVP Scope: typed serde-backed policy structs, override precedence, incident guidance selection.
- v1 Scope: finer-grained policy composition and validation tooling.
- v2/Research Scope: advanced policy negotiation between provider and local operator layers.
- Failure Modes: invalid override shape, contradictory policy bounds, unsupported platform clauses.
- Test Strategy: unit tests for precedence and bounds validation.

### `veil-routing`

- Responsibility: score endpoint candidates, track cooldowns, known-good reuse, local disable flags, re-enable timing, and retry budget enforcement.
- Inputs: endpoint set, state store, policy constraints, client platform, recent failures and successes.
- Outputs: ordered candidate list, `ScoreBreakdown`, route selection explanation, updated route state.
- Public Interfaces: `RouteSelector`, `RouteStateStore`, `ScoreBreakdown`, `RetryBudget`, `CooldownState`.
- Dependencies: `veil-policy`, persistent local state, diagnostics model types.
- Hot-swappable Parts: scoring heuristics are configurable; route-state persistence backend can vary.
- MVP Scope: current GMvpn semantics for cooldown, last-known-good, pending re-enable, and health score ordering.
- v1 Scope: richer regional/network-cost preferences and more explicit route explainability.
- v2/Research Scope: cross-session learned routing or remote reputation feeds.
- Failure Modes: unstable scoring causing thrash, stale disable flags, mismatch between candidate order and explanation output.
- Test Strategy: deterministic routing tests with score-breakdown assertions.

### `veil-diagnostics`

- Responsibility: generate incident summaries, redact support bundles, expose explainable route and runtime decisions, and preserve release-facing operator evidence.
- Inputs: session reports, route selection breakdowns, adapter runtime snapshots, local incident flags, recovery reports.
- Outputs: support bundle payloads, incident narratives, telemetry-safe records, operator summaries.
- Public Interfaces: `IncidentReporter`, `SupportBundleExporter`, `RedactionPolicy`.
- Dependencies: `veil-core`, `veil-routing`, `veil-policy`, `veil-adapter-api`.
- Hot-swappable Parts: storage/export target may vary; redaction policy remains core-owned.
- MVP Scope: support bundle and incident summary parity with the GMvpn reference semantics, plus redacted manifest, route, policy, and backend-preflight diagnostics views that are safe to show in local CLI output by default.
- v1 Scope: bounded structured diagnostics packs and better correlation IDs.
- v2/Research Scope: fleet-facing aggregation or external support integrations.
- Failure Modes: leaking sensitive fields, drift between CLI output and bundle contents, truncated evidence hiding cause of failure.
- Test Strategy: support bundle redaction and artifact parity tests.

### `veil-adapter-api`

- Responsibility: define the in-process trait boundary between control plane and dataplane/transport/platform adapters.
- Inputs: normalized endpoint config, generated runtime config, runtime context, lifecycle commands from `veil-core`.
- Outputs: adapter status, health reports, runtime snapshots, start/stop/reload results.
- Public Interfaces: `DataplaneBackend`, `TransportAdapter`, `EndpointProvider`, `HealthProbe`, `PolicyEvaluator`, `AdapterCapabilities`, `AdapterRegistrySnapshot`.
- Dependencies: shared model types only; no direct dependency on a concrete backend.
- Hot-swappable Parts: all backend and transport implementations.
- MVP Scope: static registration, lifecycle methods `init`, `build_dry_run_preflight`, `apply_config`, `start`, `health_check`, `reload`, `stop`, `runtime_snapshot`, and operator-facing capability snapshots for registered adapters.
- v1 Scope: version compatibility metadata and stronger capability negotiation.
- v2/Research Scope: sandboxed or separately isolated plugin processes.
- Failure Modes: partially applied config, health-check contract mismatch, adapter reporting incomplete runtime state.
- Test Strategy: contract tests with mock adapters.

### `veil-adapter-xray`

- Responsibility: implement the first real dataplane backend by compiling Veil endpoint/runtime models into Xray config and supervising Xray process lifecycle.
- Inputs: typed Xray-capable endpoint config, generated runtime directives, platform/network context.
- Outputs: rendered Xray config, start command, preflight results, runtime snapshot, health results.
- Public Interfaces: `XrayBackend`, `XrayConfigRenderer`, `XrayCapabilityDescriptor`.
- Dependencies: `veil-adapter-api`, process runner abstraction, filesystem/config writer.
- Hot-swappable Parts: binary path, config validation runner, renderer options.
- MVP Scope: dry-run mode, typed config rendering, command-spec building separate from lifecycle handling, binary presence checks, config preflight validation, runtime snapshot.
- v1 Scope: richer feature coverage and improved error attribution.
- v2/Research Scope: alternative renderer targets or bridge contracts for non-process runtimes.
- Failure Modes: unsupported endpoint metadata, invalid generated config, missing binary, crash after start.
- Test Strategy: renderer and dry-run adapter tests plus smoke fixtures.

### `veil-adapter-mock`

- Responsibility: provide a second safe demo backend for exercising adapter registration, dry-run preflight, and control-plane selection logic without backend-specific assumptions.
- Inputs: normalized endpoint config that opts into `mock-backend`, dry-run context, local runtime paths.
- Outputs: rendered mock config, dry-run command preview, runtime snapshot, health result.
- Public Interfaces: `MockDryRunBackend`.
- Dependencies: `veil-adapter-api`, typed manifest endpoint model.
- Hot-swappable Parts: command args and rendered payload shape may change independently of the control plane.
- MVP Scope: dry-run config rendering and preflight coverage for a non-Xray backend.
- v1 Scope: richer fake capability descriptors for integration tests.
- v2/Research Scope: dedicated harness adapters for regression suites and compatibility matrices.
- Failure Modes: unsupported endpoint selection, config rendering drift from adapter contract, test-only adapter leaking into user-facing claims.
- Test Strategy: deterministic unit tests that confirm the generic adapter boundary works across more than one backend.

### `veil-cli`

- Responsibility: provide local operator entrypoints for dry-run, runtime startup, support-bundle export, and guardrail-friendly validation.
- Inputs: manifest paths, public keys, local override files, runtime flags, cache/state paths.
- Outputs: human-readable execution summary, support bundle, exit code, validation failures.
- Public Interfaces: command-line commands, dry-run/reporting output, backend preflight export, local validation subcommands.
- Dependencies: every core module, but no backend-specific logic beyond adapter selection.
- Hot-swappable Parts: output formatters and command surface can evolve independently of control-plane logic.
- MVP Scope: dry-run and report-focused local CLI with safe redacted output by default, file-based policy overrides, backend preflight visibility, standalone redacted preflight export, and explicit opt-in raw JSON output for deeper debugging.
- v1 Scope: subcommand-oriented UX and richer diagnostics commands.
- v2/Research Scope: local API server or GUI integration shell.
- Failure Modes: CLI output drifting from support bundle, contract mismatch hidden by permissive flags, local override precedence confusion.
- Test Strategy: CLI output tests and artifact parity tests.

## Failure Modes

System-level failure classes that Veil must model explicitly:

- manifest trust failure
- compatibility/version failure
- route selection thrash
- adapter startup failure
- post-start adapter crash
- platform/runtime contract mismatch
- stale runtime recovery failure
- diagnostics redaction failure
- support-bundle/reporting drift

Design rule:

- every failure that affects routing or runtime behavior must be representable as structured state plus human-readable explanation

## Validation and Test Strategy

Required MVP checks:

- manifest signature and schema compatibility tests
- provider-profile compiler tests
- route scoring/cooldown/known-good tests
- session lifecycle and startup recovery tests
- adapter contract tests with mocks
- Xray renderer and preflight smoke tests
- support bundle redaction and parity tests, including manifest, route, and policy diagnostics views

Target command set once the Rust prototype exists in a Rust-enabled environment:

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace`

## MVP / v1 / v2 Boundaries

### MVP

- Rust prototype workspace for `veil-core`, `veil-manifest`, `veil-policy`, `veil-routing`, `veil-diagnostics`, `veil-adapter-api`, `veil-adapter-xray`, and `veil-cli`
- Linux-first runtime contour
- Xray as first backend adapter
- signed manifests and provider-profile compatibility
- explainable route selection and diagnostics parity
- redacted manifest, route, and policy diagnostics surfaces in the local support bundle
- report-focused local CLI with redacted default output and explicit raw JSON opt-in
- static in-process adapter registration

### v1

- stronger capability negotiation between manifest and adapters
- more complete desktop/Android runtime contracts
- better diagnostics packaging and redaction rules
- improved typed adapter config families

### v2 / Research

- dynamic plugin/process isolation
- broader backend portfolio
- richer cross-session learning or remote scoring inputs
- full non-Linux production hardening

## Open Questions

1. Should `veil-routing` own persistent route state directly, or should `veil-core` own the store and pass a narrower interface?
2. How much of the current policy surface should become strict typed schema in MVP versus remain internal-only until stabilized?
3. What is the smallest typed Xray config model that preserves current GMvpn behavior without recreating raw backend JSON as the main public UX?
