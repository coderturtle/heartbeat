# Agent Context: Heartbeat

Learn distributed systems the agent-native way: build `Checkout`, a distributed lock/session-ownership service, from scratch in Rust, from RPC through leader election, log replication, persistence, and sharding, mirroring MIT 6.824/6.5840's lab progression, with a `turmoil`-based network-simulation test harness as the deterministic gate and Coachgremlin grading the conceptual layer on top.

## Working in this repo

- Work on a short-lived branch; never commit directly to `main`.
- Run the verification entry point before opening a PR (fixed 2026-08-30, `docs/completion-roadmap.md` finding F: the previous instruction named `scripts/verify-project.sh`, which is gitignored and doesn't exist in a fresh public clone):
  ```bash
  bash scripts/check-prereqs.sh
  bash scripts/check-brand-lint.sh
  cd fixtures/checkout && cargo test && cargo clippy --tests -- -D warnings
  ```
- Keep changes scoped to what was asked; note assumptions in the PR description.

## Conventions

- Document decisions in `docs/decisions.md`.
- Update `docs/next-actions.md` when you finish or discover work.
- Tests and docs ship with the change, not after it.

<!-- This repo is public. It is developed inside a private factory whose internal
     contracts, ledgers and vault mirror live outside this tree; nothing here depends
     on them, and this file is deliberately self-contained. -->
