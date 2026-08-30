# Completion Roadmap

Produced 2026-08-30 via a three-agent review chain (Fable drafts → Codex CLI adversarially critiques with live repo access → Opus independently re-verifies every disputed claim and reconciles), the user's established pattern for non-trivial planning decisions. Kept as a durable artifact rather than left in chat, since it corrects several real, previously-unnoticed issues in already-merged content, not just future planning.

**Read this alongside `docs/decisions.md` and `docs/next-actions.md`, which it supersedes on the specific items below.**

## New findings, verified directly, that neither Fable nor Codex caught

**A. One of Module 01's five tests is vacuous.** `partition_surfaces_as_an_error_not_a_hang` only asserts that `TcpStream::connect` fails across a standing partition — it never calls `send_request`/`handle_one`, so it passes against the shipped `todo!()` stubs with zero implementation. Confirmed by actually running `cargo test` against the stub state: `1 passed; 4 failed` (only the vacuous one passes). This is a real bug in already-merged, "real" content. **Fix before touching this test file again.**

**B. No Rust toolchain pin, contradicting the repo's own stated policy.** `docs/workshop-design.md` point 10 says full reproducibility depends on the toolchain version, not just `Cargo.lock`. Module 01's entire dry-run finding (clippy's `unused_io_amount` catching the naive attempt) is a toolchain-versioned lint behavior, and nothing pins it. Add `rust-toolchain.toml` to `fixtures/checkout/`. Cheapest high-value fix in the whole plan.

**C. The Coachgremlin ARB structural-regression gate is unimplemented.** Heartbeat is a shared-throughline-project workshop (`Checkout` spans modules), which is exactly the case `coachgremlin.md`'s Workflow step 0 requires an ARB check for. `scripts/arb-trigger-check.sh` doesn't exist here (it does in `borrow-native`) and `.hekton/governance.yaml` has no `arb_review_triggers` block. The mandatory gate for this workshop's own shape is simply missing.

**D. There is no plan for the cumulative reference implementation — the biggest hole in both agents' plans.** The dry-run discipline requires a correct implementation per module, but Modules 03-08 are cumulative on the learner's own prior code (unlike `borrow-native`'s independent per-module functions). A working Raft + Checkout reference has to live somewhere private and be maintained across modules. Neither Fable nor Codex priced this at all — it's the dominant cost item.

**E. No defined learner resume path.** Module 05 is unattemptable without a working Module 04; Module 09's gate presupposes one cumulative working program. A learner who stalls at Module 04 has no defined way forward, and whether per-module checkpoints get published (which would double as answer keys) is an undecided curriculum question that has to be made before Module 03 ships.

**F. The public repo's own contributor instructions are broken.** `AGENTS.md` tells contributors to run `scripts/verify-project.sh`, which doesn't exist in a fresh public clone (it's gitignored; the doc it checks for lives in the private sibling). A fresh clone cannot follow its own setup instructions.

**G. `docs/operating-model.md` is an empty template**, not a document to amend — worth knowing before writing the auto-merge policy into it.

## Corrections to specific claims (both agents got things wrong)

- **Confirmed false:** the "agent-native manifest pilot has never been built" claim (Fable). It was built and dry-run-verified in `terminal-velocity` on 2026-07-04 (`modules/03-harness-engineering/module.yaml`, `AGENT.md`, root `coachgremlin/grader.md`, `runs/2026-07-04-module-03-manifest-dry-run/`). Fable's source (`~/hekton/gremlins/workshop/workshop-gremlin.md`) is itself stale here — a factory-level write-back fix is needed independent of Heartbeat.
- **Confirmed:** `turmoil`'s filesystem-crash-consistency is gated behind an `unstable-fs` feature Heartbeat's `Cargo.toml` doesn't enable — relevant before authoring Module 05.
- **Confirmed, and worse than stated:** `turmoil::hold` is bidirectional-only and cannot combine with `partition_oneway`/`repair_oneway` — a harder constraint on Module 04's Figure-8 scenario than either agent realized.
- **Confirmed:** `rand::thread_rng` is deprecated in the `rand` version actually pinned here (renamed to `rand::rng`) — and moot anyway, since `rand` only reaches this crate transitively through `turmoil` as a dev-dependency, never available to learner code.
- **Confirmed false negative:** Codex said `scripts/check-mirror-drift.sh` doesn't exist — it does, just gitignored, which is exactly why a tracked-file-listing tool missed it. It's the wrong home for status-drift checks anyway (it's for vault-mirror drift, not content) — use `check-brand-lint.sh` instead, which is tracked.
- **Fable's KDF-based seed proposal doesn't work as specified** (Codex correct): if the learner can compute the inputs, they can grind the seeds. Replaced below with a generation-based approach that doesn't depend on secrecy.
- **Real, substantive, and correct:** the held-out-seed design assumes a trust boundary (Coachgremlin as a party separate from the learner) that doesn't exist when Coachgremlin is explicitly "a role you run yourself... not a hosted service." Neither the original docs nor Codex's defense of them actually reconciled this specific contradiction — see Phase A.8 below for the resolution.

## The phased plan

Dependencies verified against the module READMEs directly: **02 and 03 are genuinely parallel** (03's only prerequisite is 01, not 02) — a scheduling win neither agent's plan used. 03→04→05→06 is a hard chain. 02+06 block 07; 07 blocks 08; all block 09. Site deploy blocks nothing. **The reference implementation (finding D) blocks every dry run from Module 03 onward** — the dependency neither original plan drew.

### Phase A — Infrastructure and honesty debt (~3-4 sessions + 1 human gate)

1. Fix the vacuous test (finding A).
2. Add `rust-toolchain.toml` to `fixtures/checkout/` (finding B).
3. Fix the broken public-clone contributor instructions (finding F).
4. Wire the ARB gate: port `borrow-native/scripts/arb-trigger-check.sh`, add `arb_review_triggers` to `.hekton/governance.yaml` (finding C).
5. Housekeeping: delete `docs/next-actions.md.bak.20260829030323`; add status-callout drift checking to `check-brand-lint.sh` (tracked), not `check-mirror-drift.sh` (gitignored, wrong purpose); write the auto-merge default into `docs/operating-model.md` (an empty template, not an amendment).
6. **Human gate, request now:** enable GitHub Pages, run `workflow_dispatch` once, confirm, add the `push` trigger. Blocks nothing; there's real content to publish already.
7. Add a Rust CI workflow (`cargo test` + `cargo clippy --tests -- -D warnings` on the pinned toolchain) — becomes ARB step 0's mechanical carrier.
8. **Decide the held-out-seed question with an actual decision:** rename "held-out" honestly (there's no second party — adopt `terminal-velocity`'s own wording: "my agent ran the grader persona locally," not "the workshop graded me"). Replace secrecy with generation: draw N seeds at submission time from a published generator over the named fault dimensions, keyed by a learner-chosen nonce — an unbounded generator can't be pre-tuned against, which is what the design doc's own point 8 upgrade path already gestured at. **This retires the "expand to >=50 fixed seeds" next-action in its current form** — build a generator plus a draw-count floor, not two fixed sets. Run one DDD cycle on this before committing.
9. Genericize the Module 01 harness into a scenario × seed matrix (not a Module 02 blocker — do it here since this phase already touches the file).

### Phase B — Reference implementation (~4-6 sessions) — the phase both agents omitted

Build a complete, correct `Checkout`-on-Raft in `heartbeat-private/`. Decide the learner-resume-path question (finding E) as part of this, since it determines what (if anything) gets published as a checkpoint.

### Phase C — Module 02 (~1.5-2 sessions), parallel with Phase B

First real `Checkout` logic. Where gate rule 6 first needs mechanical enforcement (clippy `disallowed-methods`, see below). Dry run re-checks the clippy-generalization question fresh.

### Phase D — Raft core, Modules 03-06 (~9-12 sessions)

Before 03: one DDD cycle on the fixture API split (provided vs. learner-written, message types, the test-observability contract for "at most one leader per term," continuously). Module 04 needs scripted scenario choreography, constrained by `hold`'s bidirectional-only, no-oneway-combination limits (verified above) — design against that constraint or state it can't be built with 0.7.2. Module 05 must resolve the `unstable-fs` question before authoring. Review Panel run #2 after Module 03, #3 after Module 06.

### Phase E — Module 07 (~3-4 sessions)

Pre-authoring DDD cycle on linearizability checking. Module 06's own README already states the mid-lease and freed-resource-generation snapshot scenarios as Module 07's dry-run checklist — use verbatim. Deliver the Module 06→07 debugging aid here.

### Phase F — madsim spike + Module 08 (~0.5 + 4-5 sessions)

Timeboxed spike first (two 3-host groups, inter-group partition, simulated migration handoff) before deciding madsim vs. turmoil, on evidence. DDD the module's own scope — 6.5840's largest lab, may need splitting.

### Phase G — Module 09 (~2-3 sessions + 1 grading DDD)

Capstone grading inherits Phase A.8's reframe: the honest bar is a reproducible grading transcript a third party can re-run, not an unintrospectable directory (same ownership problem as held-out seeds). Review Panel run #4, then final status sweep.

**Total effort: ~28-38 sessions**, dominated by Phase B's uncertainty (building a correct reference Raft implementation is notoriously bimodal in effort). Multi-month at typical session cadence.

## New approaches — verdicts

**Pursue:**
1. `clippy.toml` `disallowed-methods`/`disallowed-types` — real for the time-related methods (`SystemTime::now`, `Instant::now`) and the `HashMap`→`BTreeMap` swap; the `rand` half is nearly moot since rand is a transitive dev-dependency only, never reachable from learner code.
2. `cargo-mutants` as a validation spike — a real reference solution already exists as `runs/2026-08-29-module-01-dry-run/attempt-good/diff.patch`, so this is more actionable than either agent thought. Finding A (the vacuous test) is itself the strongest argument for doing this.
3. Generation-based grading seeds (the honest replacement for Fable's KDF idea) — see Phase A.8.
4. Text-first seed-replay trace timeline (JSON-lines event log + a lane renderer) as the Module 06→07 debugging aid. Defer the HTML/SVG visualizer.
5. A bespoke linearizability checker for Module 07 — Checkout's spec is small enough to make this tractable, and it's good teaching material in its own right.
6. Underused `turmoil` capabilities (`hold`/`release`, `crash`/`bounce`) for Modules 04-05 — with the real constraints now documented above.
7. The agent-native manifest pilot — now confirmed cheaper than originally thought, since a working, validated reference already exists in `terminal-velocity` to port against. Do it for Module 01 first.
8. The real-network localhost epilogue after Module 07 — genuinely motivating, but only cheap relative to Module 07's own cost, not "free."

**Against:** madsim migration (only if the Phase F spike fails), stateright/TLA+ (wrong workshop, displaces the turmoil thesis), an MCP grading server (resurrects the secrecy model Phase A.8 just retired), an HTML cluster visualizer (v2, behind idea 4).

## What's still unverified

GitHub Pages' actual settings state (not filesystem-checkable); the current `npm audit` finding count (not re-run, only the cross-workshop pin match was verified); `cargo-mutants`' behavior against turmoil-heavy integration tests and its wall-clock-timeout tension with this workshop's own no-wall-clock gate rule; whether `unstable-fs` is actually required for Module 05 or `Sim::crash`/`bounce` alone suffice; and the session-count estimates, calibrated off one module's worth of real data.
