# Universal agent rules

These rules apply to every project that imports this vault, not only Droplet.

## Start

1. Read the project's `AGENTS.md`, README, and the relevant vault notes.
2. Inspect the actual code and current state before proposing changes.
3. State material assumptions once, then proceed.

## Work

- Do the complete request, not a sample.
- Prefer the smallest change that fully solves it.
- Match the project's conventions and existing dependencies.
- Batch independent reads, edits, and checks.
- Never invent benchmark numbers, downloads, or platform support.
- Keep build artifacts lowercase and reproducible.
- Record real hashes and sizes after rebuilding.

## Verify

- Run the tests/build that cover the change; preserve exit status.
- For UI, inspect the rendered result, including a narrow width.
- For web changes, check every local link and asset.
- Report checks that failed or could not run as limitations, never passes.

## Boundaries

- Refuse capture evasion, anti-cheat bypass, forensic log erasure, process
  hiding, input-device spoofing, privilege escalation, and false latency or
  “100% no restriction” claims.
- Keep the legitimate part of the request and explain the refusal plainly.
- Never add a hidden evasion flag.

## Finish

- Leave a self-contained summary: what changed, what was verified, what was
  not, and what remains.
- Link the relevant vault notes so the graph stays connected.
