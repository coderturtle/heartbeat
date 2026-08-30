# Operating Model: Heartbeat

## Classification

**factory-output** — promotion target: `none`

## How to Run

This is a workshop repo, not a running service — there's nothing to "run" in the deploy sense. A learner works through `modules/` in order, implementing each module's exercise inside their own coding-agent harness. The build-log site (`site/`) is a static Astro build, deployed to GitHub Pages once the human-confirmed first deploy happens (see `docs/next-actions.md`).

## Setup

```bash
git clone git@github.com:coderturtle/heartbeat.git
cd heartbeat
bash scripts/check-prereqs.sh
cd fixtures/checkout && cargo test  # confirms the toolchain and deps resolve
```

Rust and `cargo` via `rustup` are required (see `README.md`'s Prerequisites); `fixtures/checkout/rust-toolchain.toml` pins the exact version.

## Key Commands

| Command | What it does |
|---|---|
| `bash scripts/check-prereqs.sh` | Confirms basic shell tooling is present |
| `bash scripts/check-brand-lint.sh` | Checks published content against `docs/brand.md`'s hard rules |
| `bash scripts/arb-trigger-check.sh --dry-run` | Checks pending changes against `.hekton/governance.yaml`'s `arb_review_triggers` — run before touching `fixtures/checkout/src/*` (Coachgremlin Workflow step 0) |
| `cd fixtures/checkout && cargo test` | Runs a module's deterministic-tier test suite |
| `cd fixtures/checkout && cargo clippy --tests -- -D warnings` | Runs the other half of the deterministic tier — see Module 01's own dry run for why this isn't redundant with `cargo test` |
| `cd site && npm run build && npx astro check` | Builds and type-checks the build-log site |

## Maintenance

- Update `docs/session-log.md` after every material session
- Update `docs/decisions.md` for durable design decisions
- Update `docs/risks.md` and `.hekton/risk-register.yaml` when risk changes
- Update `docs/next-actions.md` when the work queue changes
- Keep `docs/project-walkthrough.md` current for human understanding
- Run `scripts/governance-check.sh` where available before handoff

## Agent-generated code: review policy

Decided 2026-08-30 (`docs/completion-roadmap.md`, resolving an open item from the Workshop Review Panel's Security-Conscious Reviewer persona): **a passing deterministic gate never auto-merges agent-generated code.** A human reviews the diff before it lands, same as any other PR in this repo — this applies to Coachgremlin's own module-authoring work as much as to a learner's exercise attempt. `cargo test`/`cargo clippy` passing is necessary evidence for a PR to be mergeable, never sufficient on its own; this is the same discipline `docs/workshop-design.md`'s own gate design already applies to the *product's* correctness claims, extended to the *process* that ships this workshop's own content.

## Upgrade / Deprecation Path

Cross-workshop shared surfaces (the `astro`/`@astrojs/*` pin in `site/`, the Astro-on-Pages publishing pipeline itself) are shared with `borrow-native` and `terminal-velocity` — an upgrade decision for one is a candidate decision for all three, not a heartbeat-only call. See `docs/next-actions.md` for the currently-deferred `astro` version bump and its stated reason.

## Known Issues

See `docs/completion-roadmap.md` for the full, current list of open structural questions (the cumulative-reference-implementation question, the learner-resume-path question) — not duplicated here to avoid two copies drifting apart. The held-out-seed grading model's trust-boundary contradiction (Phase A.8) is resolved — see `docs/workshop-design.md`'s deterministic-gate section, point 2.

