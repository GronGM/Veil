# Veil

Veil is an adaptive VPN/proxy orchestration platform.

The goal of the project is to build a control plane that can take trusted provider inputs, local policy, runtime health, and adapter capabilities, then turn that into explainable connection decisions, generated runtime configuration, and supportable operational behavior.

Veil is not meant to be a thin launcher around one backend and not a project built around hand-editing backend-native JSON as the main user workflow. The product value is supposed to live in orchestration, validation, routing, diagnostics, and clear runtime boundaries.

## What This Project Is About

Veil is being designed as a system where:

- the control plane is separate from the dataplane
- dataplane backends are replaceable adapters
- transport behavior is replaceable
- provider and user inputs are expressed as higher-level profiles and policy
- route selection, failover, and health decisions are observable and explainable
- diagnostics and support output are first-class instead of an afterthought

In practical terms, Veil is trying to become the layer that decides:

- which backend should be used
- which route should be selected
- when a route should be rejected or cooled down
- how runtime configuration should be generated
- how incidents should be explained to operators

## Why Veil Exists

Many VPN/proxy stacks are strong at backend execution but weak at orchestration.

That usually means one or more of the following:

- backend-specific configuration leaks directly into the user-facing workflow
- routing and failover decisions are hard to inspect
- support and incident handling are too opaque
- one backend becomes the architecture by accident

Veil exists to solve that class of problem.

The project is intended to provide a clearer orchestration model for VPN/proxy runtimes, where trust, policy, routing, diagnostics, and backend lifecycle all have explicit ownership.

## What We Are Building Toward

The long-term direction for Veil is:

- a signed-manifest and provider-profile driven control plane
- typed policy evaluation instead of ad hoc runtime behavior
- explainable route scoring, cooldown, retry, and fallback behavior
- replaceable backend adapters instead of one hardcoded engine
- redacted diagnostics and support bundles that are actually useful in operations
- a modular Rust-priority implementation for the core system

## Intended MVP

The first honest MVP should provide:

- a control-plane-first architecture
- one real runtime contour instead of broad unsupported parity claims
- manifest and compatibility validation
- explainable routing and failover behavior
- adapter boundaries that make backend replacement possible later
- diagnostics that help operators understand what happened and why

The point of the MVP is not to claim that Veil solves every platform and backend problem immediately. The point is to establish a reliable foundation with honest boundaries.

## What Veil Is Not Claiming

Veil is not currently meant to imply:

- full production readiness
- full cross-platform parity
- dynamic plugin loading as an MVP requirement
- invisible, undetectable, or censorship-proof behavior
- that one backend should define the whole architecture

## Design Principles

The project is being shaped around a few durable principles:

- preserve clear separation between control plane and dataplane
- prefer replaceable adapters over backend lock-in
- treat runtime configs as generated artifacts, not the main authoring surface
- make routing and failure handling explainable
- keep security claims measurable and bounded
- avoid blind rewrites and oversized implementation jumps

## Current State

Veil is still early in its public repository shape.

This repository is intended to become the home for the project's architecture, prototype implementation, and phased development work. At this stage, the most important thing is making the project direction legible: what we are building, why it matters, and what a realistic destination looks like.

The public branch now also includes the first `prototypes/veil-rust/` workspace skeleton so future implementation slices have a visible home.

## Repository Layout

- `README.md` - project overview and goals
- `docs/veil-architecture-rfc.md` - initial architecture direction
- `docs/veil-phase-map.md` - public phase map and delivery shape
- `prototypes/README.md` - expectations for future prototype work
- `prototypes/veil-rust/` - initial public Rust workspace skeleton

## Bottom Line

Veil is a control-plane-first effort to build a modular, explainable, supportable VPN/proxy orchestration platform.

The project is aiming for a system that can safely ingest trusted inputs, generate runtime behavior through replaceable adapters, explain its routing and failover decisions, and give operators something they can actually understand and support.
