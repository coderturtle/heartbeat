# Next Actions: Heartbeat

## Immediate

- [x] Naming pass complete — Heartbeat
- [x] `docs/workshop-design.md` drafted (curriculum anchor, module arc, gate-harness decision)
- [x] Run first Workshop Review Panel pass against naming + design docs
- [x] Build module/deliverables skeleton (`modules/01-...` through `09-synthesis-capstone`) + brand layer (`docs/brand.md`) + README rework + `docs/maintainers.md` split
- [x] Stand up build-log/Pages site skeleton — `site/` (Astro 5, Content Layer API reading `docs/build-log/`), one real seed entry (`docs/build-log/2026-08-29-scaffold-and-design.md`), `.github/workflows/deploy-pages.yml` (`workflow_dispatch` only for now). `npm run build` and `npx astro check` both clean locally; `npm audit` found 5 inherited-from-starter vulnerabilities (astro/esbuild/sharp, fixable only via a major astro version bump — flagged, not applied unilaterally, see `docs/decisions.md`)
- [x] Revise the deterministic-gate design through 3 doubt-driven-development cycles (see `docs/decisions.md` and `docs/workshop-design.md`)
- [x] Reseed the arc around one real product, `Checkout` (see `docs/decisions.md` and `docs/workshop-design.md`'s "The shared project" section)

## Superseded, 2026-08-30

The lists that used to live in "This Week" and "Later" below are superseded by **`docs/completion-roadmap.md`**, produced via a three-agent review chain (Fable plan → Codex critique → Opus reconciliation) that independently re-verified the repo state and found several real issues the prior list didn't capture (a vacuous test, a missing toolchain pin, an unimplemented ARB gate, no plan for the cumulative reference implementation, a design contradiction in the held-out-seed model). Read that doc for the authoritative phased plan. This file now tracks only the immediate Phase A items in flight.

## This Week (Phase A of `docs/completion-roadmap.md`)

- [x] Author Module 01 (Coachgremlin's first real content-building pass): exercise, test suite, rubric, and real dry run complete (`runs/2026-08-29-module-01-dry-run/`)
- [ ] Fix `partition_surfaces_as_an_error_not_a_hang` — currently vacuous, passes against unimplemented `todo!()` stubs (roadmap finding A)
- [ ] Add `rust-toolchain.toml` to `fixtures/checkout/` (roadmap finding B)
- [ ] Fix `AGENTS.md`'s broken contributor setup instructions — points at a `scripts/verify-project.sh` that doesn't exist in a fresh public clone (roadmap finding F)
- [ ] Wire the Coachgremlin ARB structural-regression gate — port `borrow-native/scripts/arb-trigger-check.sh`, add `arb_review_triggers` to `.hekton/governance.yaml` (roadmap finding C)
- [ ] Decide the held-out-seed question for real (roadmap Phase A.8: rename "held-out" honestly, replace secrecy with a published seed generator + draw-count floor) — run one DDD cycle first
- [ ] Add a Rust CI workflow (`cargo test` + `cargo clippy --tests -- -D warnings`) — none exists yet
- [ ] **Human-confirmed first live GitHub Pages deploy** — blocks nothing, real content exists to publish already
- [ ] Housekeeping: delete `docs/next-actions.md.bak.20260829030323`; move status-callout drift checking into `check-brand-lint.sh` (tracked), not `check-mirror-drift.sh` (gitignored, wrong purpose); write the auto-merge default into `docs/operating-model.md` (currently an empty template)

## Later (Phases B-G of `docs/completion-roadmap.md`)

- [ ] **Phase B — build the cumulative reference Raft+Checkout implementation in `heartbeat-private/`, and decide the learner-resume-path question.** Nothing else from Module 03 onward can be honestly dry-run-validated without this; neither the original plan nor its critique priced it.
- [ ] Phase C — author Module 02
- [ ] Phase D — author Modules 03-06 (DDD the fixture API split before 03; Module 04 needs scripted scenario choreography, constrained by `turmoil::hold` being bidirectional-only; Module 05 needs the `unstable-fs` feature-flag question resolved first; Review Panel runs after 03 and 06)
- [ ] Phase E — author Module 07 (DDD the linearizability-checking approach first; deliver the Module 06→07 debugging aid)
- [ ] Phase F — madsim spike, then author Module 08
- [ ] Phase G — author Module 09 (capstone), Review Panel run #4, final status sweep
- [ ] Update `~/hekton/gremlins/workshop/workshop-gremlin.md`'s stale "agent-native manifest pilot, never built" claim — it was built and dry-run-verified in `terminal-velocity` on 2026-07-04; a factory-level fix independent of Heartbeat
