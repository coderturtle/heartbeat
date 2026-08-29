# Maintainers

This is the internal/agent-facing doc. Learners should read the top-level `README.md` instead; this file is for anyone working on the workshop itself.

**Classification:** factory-output
**Lifecycle:** active
**Owner:** coderturtle
**Promotion target:** `none`

This repo has two goals:

1. **Ship a workshop** teaching distributed systems (RPC, single-node service design, leader election, log replication, persistence, log compaction, replicated state machines, sharding) to agent-literate practitioners already fluent in Rust, taught by running every exercise through a real harness with `turmoil`-simulated network faults as the deterministic gate and Coachgremlin grading the conceptual layer on top.
2. **Feed evidence back into the reusable machinery**: this is the Workshop Gremlin's fourth real run (`terminal-velocity` first, `borrow-native` second, `half-life` third) and its first on a subject with a genuinely different deterministic-gate shape — a designed network-fault simulator (`turmoil`) rather than a compiler/linter (`borrow-native`) or a designed empirical experiment (`half-life`). Any generalizable finding here should be written back into `<hekton-machinery>/gremlins/workshop/workshop-gremlin.md`.

## Implementation Status

- 2026-08-29 — Scaffolded as factory-output (as `distributed-systems-workshop`). Naming pass complete: **Heartbeat**. `docs/workshop-design.md` complete (audience, deterministic-gate method, MIT 6.5840 curriculum-anchor research, 9-module arc). First [Workshop Review Panel](review-panel/2026-08-29-initial-design.md) pass complete against the naming + design docs — all seven personas returned distinct findings; design-doc-fixable ones applied in the same pass (most substantive: folding multi-seed `turmoil` testing into the deterministic tier itself).
- Module skeleton (`modules/`), brand layer (`docs/brand.md`), and this maintainers split are done. Build-log/Pages site is the remaining Completion Condition item — see [Next Actions](next-actions.md).

## Documentation Contract

Agents working here must inspect `.hekton/project.yaml` before structural changes, keep `docs/session-log.md` current, record meaningful design decisions in `docs/decisions.md`, and update `docs/next-actions.md` when the work queue changes.

Vault mutation policy: see `vault_mutation_allowed` in `.hekton/project.yaml` (authoritative; defaults to false at scaffold time). The repo-local `mind-palace/` folder is only a mirror draft; do not write to the live vault unless `.hekton/project.yaml` says mutation is allowed and it is explicitly authorised in-session.

## Voice and style for published content

Anything a learner reads (README, module content, build-log entries, the site once built) follows `docs/brand.md` — voice, hard rules (no em dashes, no unqualified efficacy claims, no framing a passing deterministic gate as proof of correctness), banned phrases. Internal docs under `docs/` are working documents and are exempt.

## Key Docs

- [Workshop Design](workshop-design.md) — audience, format, MIT 6.5840 curriculum research, deterministic-gate teaching method, module arc
- [Brand / Style Layer](brand.md) — voice, hard rules, visual identity
- [Workshop Review Panel Report](review-panel/2026-08-29-initial-design.md) — 7-persona critique of the naming + design docs, first run
- [Modules index](../modules/README.md) — the full arc, gate tiers, and per-module skeleton status
- [Session Log](session-log.md)
- [Decisions](decisions.md)
- [Risks](risks.md)
- [Project Walkthrough](project-walkthrough.md)
- [Next Actions](next-actions.md)
- [Operating Model](operating-model.md)
- [Human Understanding Check](human-understanding-check.md)
- [Depth Decision](depth-decision.md)
- [Retire / Promote Review](retire-promote-review.md)
