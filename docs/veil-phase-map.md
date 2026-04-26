# Veil Phase Map

## Purpose

This file is the public phase map for Veil.

It explains how the project should move from a clear idea into a reviewable implementation without drifting into a blind rewrite or oversized undocumented changes.

## Current Public Reading

The public `main` branch is still in the early project-structuring stage.

That means:

- the README explains the project direction
- the architecture RFC defines the intended shape
- the repository is not yet claiming a completed prototype on this branch

## Phase 0: Public Framing And Repository Grounding

Goal:
Make the project legible to new readers and establish the basic architectural direction.

Expected outputs:

- meaningful README
- initial architecture RFC
- initial phase map
- clear statement of goals, boundaries, and non-goals

Exit criteria:

- a newcomer can understand what Veil is trying to build
- the repo has a documented architecture direction
- the project does not overclaim current implementation maturity

## Phase 1: Repository Intake And Reference Mapping

Goal:
Ground Veil design work in the existing GMvpn reference system instead of in abstract guesses.

Expected outputs:

- repository-grounded audit
- responsibility map from reference modules into Veil modules
- initial risk register

Exit criteria:

- reference behavior is mapped before major implementation work begins
- control-plane and dataplane responsibilities are described clearly
- the project avoids a blind rewrite posture

## Phase 2: Rust Workspace Skeleton

Goal:
Create a clean prototype workspace that reflects the intended module boundaries.

Expected outputs:

- Cargo workspace
- initial crate layout
- prototype README and validation notes

Exit criteria:

- the workspace structure is reviewable
- module names match the architecture docs closely enough to understand the plan
- limitations are documented honestly

## Phase 3: Adapter And Control-Plane Contracts

Goal:
Prove the boundary between orchestration and backend-specific behavior.

Expected outputs:

- adapter API contracts
- first backend adapter skeleton
- dry-run oriented CLI and validation path

Exit criteria:

- one backend adapter can be described and exercised through a stable contract
- the architecture no longer depends on one backend-specific control path
- docs explain what is implemented versus what is still planned

## Phase 4: Manifest, Policy, And Routing Hardening

Goal:
Add typed trust, policy, and routing behavior with explainable outcomes.

Expected outputs:

- typed manifest handling
- typed policy structures
- explainable route scoring and fallback behavior
- diagnostics shaped around operator understanding

Exit criteria:

- key control-plane decisions are explicit and testable
- routing and failure behavior are explainable
- docs remain honest about MVP boundaries

## Phase 5: Diagnostics, Threat Model, And Release Guardrails

Goal:
Turn the project from a promising prototype into something with disciplined operational boundaries.

Expected outputs:

- diagnostics and support-bundle rules
- threat model
- release guardrails
- validation checklist

Exit criteria:

- the project has clear operational and security boundaries
- release claims are tied to real validation
- public docs describe the actual state of readiness

## Working Rule

Each phase should land in small, reviewable slices.

The repo should prefer:

- documentation before ambiguity spreads
- clear module boundaries before backend sprawl
- explicit validation before stronger claims
