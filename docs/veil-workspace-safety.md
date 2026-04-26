# Veil Workspace Safety Notes

This note exists to reduce the risk of accidentally leaking local-only workspace files into the Veil repository.

## Scope

These rules apply to local artifacts that may appear around the repository during prototype work, agent-assisted work, or local review workflows.

Examples:

- workspace-provided `agent_files/`
- user-supplied `user_files/`
- generated `output/`
- ad hoc local secrets, credentials, or private keys

## Rules

- Do not commit workspace-provided files such as `agent_files/` into the repository.
- Do not copy token, credential, or secret-like files from the workspace into `docs/`, `prototypes/`, tests, fixtures, or examples.
- Do not include real secrets in diagnostics output, redacted support artifacts, examples, or screenshots.
- Treat any local token or credential file as out-of-scope operational data unless a phase explicitly requires a reviewed secret-handling task.
- Prefer mock values, placeholders, and clearly fake demo material in prototype flows.

## Current Known Risk

During the current workspace session, a secret-like token file exists outside the Veil repository at:

- `/workspace/agent_files/tokenveil.txt`

That file is not part of Veil source code and should not be moved, copied, or referenced into repository artifacts by default.

## Repository Hygiene

The repository now ignores several common local-only paths and secret-like file patterns, but ignore rules are only a guardrail.

They are not a substitute for deliberate review before:

- creating commits
- exporting artifacts
- opening pull requests
- publishing diagnostics or support bundles

## Export And Review Boundary

Treat exported artifacts as publishable by default.

That means:

- do not export raw workspace snapshots when a focused report or generated artifact is enough
- review example configs, diagnostics, and screenshots for copied local paths or secret-like values
- prefer redacted, fake, or clearly placeholder material in docs and demos

## Maintainer Guidance

Before any future publish step, confirm that no secret-like files are present in staged changes or generated deliverables.

If a real secret is accidentally exposed in a workspace file, rotate it through the appropriate provider workflow instead of treating repository cleanup alone as sufficient remediation.
