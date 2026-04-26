# Veil Architecture RFC

## Overview

This document defines the initial public architecture direction for Veil.

Veil is being built as an adaptive VPN/proxy orchestration platform. The project focus is the control plane: trust, policy, routing, diagnostics, and adapter lifecycle. Dataplane engines matter, but they are not supposed to become the whole product architecture.

## Problem Statement

Many VPN/proxy systems are difficult to evolve and difficult to support because:

- one backend-specific format becomes the real UX
- routing and failover logic is buried in backend or launcher behavior
- support outputs do not explain why a decision was made
- extending the system means coupling more logic to one engine instead of to clear interfaces

Veil exists to solve that class of problem by making orchestration explicit.

## Architectural Direction

The core direction for Veil is:

- control-plane first
- dataplane backends are replaceable adapters
- transport behavior is replaceable
- runtime configs are generated artifacts, not the primary authoring surface
- routing, failover, and health decisions must be explainable
- diagnostics must be useful in real operator workflows

## Proposed Module Map

The initial planned module boundaries are:

- `veil-core`: orchestration and runtime lifecycle coordination
- `veil-manifest`: trusted provider input, compatibility, and profile handling
- `veil-policy`: typed policy resolution and override precedence
- `veil-routing`: endpoint scoring, cooldown, retry, and fallback behavior
- `veil-diagnostics`: incident summaries, support bundles, and redaction rules
- `veil-adapter-api`: the control-plane boundary for dataplane and transport adapters
- `veil-adapter-xray`: the first concrete backend adapter
- `veil-cli`: local operator and validation entrypoint

These are target boundaries for phased implementation, not a claim that every module already exists on this public branch.

The public branch now includes the first `prototypes/veil-rust/` skeleton so those boundaries have a visible workspace home.

## MVP Direction

The first honest MVP should aim for:

- a control-plane-first architecture
- one real runtime contour rather than broad unsupported parity claims
- signed manifest and compatibility handling
- typed policy inputs
- explainable route selection, cooldown, and fallback behavior
- redacted diagnostics and support output
- clear backend adapter boundaries

## Non-Goals

Veil is explicitly not trying to do the following in its early phases:

- blind rewrite everything at once
- treat one backend as the architecture
- require users to hand-edit backend-native JSON as the main workflow
- promise full production readiness too early
- promise full cross-platform parity too early
- make unsupported claims about invisibility, undetectability, or censorship resistance

## Current Public State

At the time of this document:

- the public repository is still in an early structuring stage
- this RFC defines intended architecture, not completed implementation
- `prototypes/veil-rust/` now exists as a public workspace skeleton
- future prototype work should land in small, reviewable slices

## Near-Term Deliverables

The next useful public steps are:

1. establish the public phase map
2. create the first prototype workspace skeleton
3. add the initial module and adapter boundaries
4. keep documentation aligned with what the repository actually contains
