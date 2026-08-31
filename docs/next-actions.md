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
- [x] Fix `partition_surfaces_as_an_error_not_a_hang` — was vacuous, passed against unimplemented `todo!()` stubs (roadmap finding A); fixed and re-verified against all three known implementation states, see `runs/2026-08-29-module-01-dry-run/grading.md`'s 2026-08-30 addendum
- [x] Add `rust-toolchain.toml` to `fixtures/checkout/` (roadmap finding B)
- [x] Fix `AGENTS.md`'s broken contributor setup instructions (roadmap finding F)
- [x] Wire the Coachgremlin ARB structural-regression gate — `scripts/arb-trigger-check.sh` ported, `arb_review_triggers` added to `.hekton/governance.yaml` (roadmap finding C)
- [x] Decide the held-out-seed question for real (roadmap Phase A.8) — resolved via three doubt-driven-development cycles (single-model cycle 1 + Codex cross-model, single-model cycles 2-3): "held-out" retired, replaced with a grading draw derived from the solution's own git tree hash (not a plain commit hash — see `docs/workshop-design.md`'s point 2 for why that distinction matters), a fully specified transcript, and two residuals named honestly rather than claimed closed (trivial-edit grinding; a self-administered version-pin check's blindness to deliberate pin selection, mitigated by preferring CI-run transcripts for anything shown to others)
- [x] Add a Rust CI workflow — `.github/workflows/rust-ci.yml` (cargo test/clippy/audit on the pinned toolchain), merged via PR #13. **Found and fixed a real `git-guardrail.sh` gap along the way**: combined `git add`+`git commit` in one Bash call bypassed the protected-path and secret-pattern checks entirely (architecturally distinct from the earlier manifest-regex bug) — see `docs/decisions.md`'s 2026-08-30 entry and memory `git_guardrail_add_commit_timing_fix`
- [ ] **Human-confirmed first live GitHub Pages deploy** — blocks nothing, real content exists to publish already
- [x] Housekeeping: status-callout drift check added to `check-brand-lint.sh`; auto-merge policy written into `docs/operating-model.md`

## Later (Phases B-G of `docs/completion-roadmap.md`)

- [x] **Phase B — build the cumulative reference Raft+Checkout implementation in `heartbeat-private/`, and decide the learner-resume-path question.** Complete: all 8 modules (RPC layer through sharding) built, tested, and tagged in `heartbeat-private/` (never merged/published here, per the Phase A.8 resume-path decision). Module 08 (sharding) took four doubt-driven-development cycles on the design before implementation - one past the process's normal 3-cycle guideline - after cycles 1-3 kept surfacing real structural bugs (an edge-triggered protocol shape that couldn't survive a leader change mid-migration was the root cause cycle 3 found; cycle 4's level-triggered redesign fixed it). Building and testing the reference implementation surfaced two genuinely valuable results beyond what design review caught: a previously-latent bug in already-tagged Module 04/06 Raft code (`on_append_entries`'s follower-side log-consistency check used `term_at` instead of `prev_term_for`, livelocking a follower that had just caught up via a real `InstallSnapshot`), and - via a retroactive DDD pass prompted by a direct question about whether implementation fixes had been reviewed - a case where an unreviewed bug fix had itself silently regressed the very safety property the four-cycle design process was built to protect. Final state: 48/48 tests pass, clippy clean, stable across 8 repeated full-suite runs. See `heartbeat-private/docs/raft-fixture-design.md` for the complete design and bug-finding history.
- [ ] Phase C — author Module 02
- [ ] Phase D — author Modules 03-06 (DDD the fixture API split before 03; Module 04 needs scripted scenario choreography, constrained by `turmoil::hold` being bidirectional-only; Module 05 needs the `unstable-fs` feature-flag question resolved first; Review Panel runs after 03 and 06)
- [ ] Phase E — author Module 07 (DDD the linearizability-checking approach first; deliver the Module 06→07 debugging aid)
- [ ] Phase F — madsim spike, then author Module 08
- [ ] Phase G — author Module 09 (capstone), Review Panel run #4, final status sweep
- [ ] Update `~/hekton/gremlins/workshop/workshop-gremlin.md`'s stale "agent-native manifest pilot, never built" claim — it was built and dry-run-verified in `terminal-velocity` on 2026-07-04; a factory-level fix independent of Heartbeat
