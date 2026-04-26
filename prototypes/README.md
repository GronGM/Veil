# Prototypes

This directory is intended for Veil prototype work.

Prototype code should exist to test architecture decisions, contracts, and validation flows before those ideas are treated as stable production surfaces.

## What Should Live Here

Examples of suitable prototype content:

- early workspace scaffolding
- adapter API experiments
- dry-run validation tools
- manifest, policy, or routing prototypes
- diagnostics and support-bundle experiments

The public branch now includes `veil-rust/` as the first visible workspace skeleton for future Rust prototype slices.

## What Should Not Happen Here

This directory should not be used to:

- pretend experimental code is production-ready
- hide large rewrites without documentation
- mix unrelated exploratory work into one oversized change

## Working Rule

Prototype changes should be:

- small
- reviewable
- documented
- explicit about limitations
