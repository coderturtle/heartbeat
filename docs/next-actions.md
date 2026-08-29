# Next Actions: Heartbeat

## Immediate

- [x] Naming pass complete — Heartbeat
- [x] `docs/workshop-design.md` drafted (curriculum anchor, module arc, gate-harness decision)
- [x] Run first Workshop Review Panel pass against naming + design docs
- [x] Build module/deliverables skeleton (`modules/01-...` through `09-synthesis-capstone`) + brand layer (`docs/brand.md`) + README rework + `docs/maintainers.md` split
- [x] Stand up build-log/Pages site skeleton — `site/` (Astro 5, Content Layer API reading `docs/build-log/`), one real seed entry (`docs/build-log/2026-08-29-scaffold-and-design.md`), `.github/workflows/deploy-pages.yml` (`workflow_dispatch` only for now). `npm run build` and `npx astro check` both clean locally; `npm audit` found 5 inherited-from-starter vulnerabilities (astro/esbuild/sharp, fixable only via a major astro version bump — flagged, not applied unilaterally, see `docs/decisions.md`)
- [x] Revise the deterministic-gate design through 3 doubt-driven-development cycles (see `docs/decisions.md` and `docs/workshop-design.md`)
- [x] Reseed the arc around one real product, `Checkout` (see `docs/decisions.md` and `docs/workshop-design.md`'s "The shared project" section)

## This Week

- [ ] Re-run the Review Panel once real module content exists (Coachgremlin's job, later run)

## Later

- [x] Author Module 01 (Coachgremlin's first real content-building pass): exercise, test suite, rubric, and real dry run complete (`runs/2026-08-29-module-01-dry-run/`) — see `docs/decisions.md` for the real finding (cargo test alone didn't catch a read()-vs-read_exact() bug; cargo clippy did)
- [ ] Author Modules 02-09 (Coachgremlin, one module at a time, with real dry runs per the existing Gremlin pattern) — check fresh each time whether Module 01's "clippy catches what tests don't" pattern generalizes, per that module's own retro
- [ ] Genericize `fixtures/checkout/tests/module_01_rpc_harness.rs`'s fault-injection harness into a reusable template once Module 02+ needs it, per Module 01's own stated takeaway
- [ ] Expand Module 01's illustrative 5-seed set to the real practice/held-out split `docs/workshop-design.md` requires (>= 50 independently-generated, disjoint seeds per set) before treating this module as gradeable, not just dry-run-validated
- [ ] Decide whether `madsim` is ever needed for Module 08 (sharding) specifically — flagged open in `docs/workshop-design.md`
- [ ] **Human-confirmed first live GitHub Pages deploy** — still open. The site skeleton and deploy workflow exist locally and build clean, but GitHub Pages has not been enabled in repo settings, and the workflow is `workflow_dispatch`-only (no `push` trigger yet) so merging the skeleton to `main` cannot itself trigger a live deploy. Enabling Pages and running `workflow_dispatch` once to confirm the site is ready to be public is a human-confirmed action per the Workshop Gremlin's own Human Gate, same as `borrow-native`'s precedent.
- [ ] Decide whether to bump `astro` (and `@astrojs/mdx`/`@astrojs/tailwind`) past the current `^5.0.0` pin to clear the inherited `npm audit` findings in `site/` — a major-version, human-confirmed dependency change per `dependency_changes: human_required`, and likely a cross-workshop decision (same pin shared with `borrow-native`/`terminal-velocity`) rather than a heartbeat-only fix
- [ ] Design a debugging aid for the Module 06→07 integration point (first time persistence, compaction, and replication all combine) — flagged by the Review Panel's End-User/Learner persona as the highest-risk drop-off point in the arc; Coachgremlin's job at content-building time
- [ ] Decide and document whether agent-generated code auto-merges on a passing gate or requires human review first — flagged by the Review Panel's Security-Conscious Reviewer persona; an implementation-spec decision, not resolved by this design pass

## Session Update: 2026-08-29 — Scaffold, name, design, review, and build skeleton for Heartbeat

- [ ] Author Modules 01-09 via Coachgremlin one at a time with real dry runs
- [ ] get human to enable GitHub Pages and trigger first deploy
- [ ] decide the astro-version-bump question
- [ ] decide whether Module 08 needs madsim instead of turmoil
