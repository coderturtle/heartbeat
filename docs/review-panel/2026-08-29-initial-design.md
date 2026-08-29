# Workshop Review Panel: Initial Design (2026-08-29)

**Input reviewed:** `docs/workshop-design.md`, `docs/decisions.md`, `docs/next-actions.md`, `README.md` (still generic scaffold boilerplate at review time).
**Panel run:** first pass, post-naming/pre-build checkpoint, per `workshop-gremlin.md`'s roster step 3. Seven personas, independent parallel fan-out, no persona saw another's output before writing its own.

## Agreements (2+ personas, independently)

1. **The gate's honesty gap is a real design flaw, not just an open risk.** The design doc's own "Open, stated honestly" section admits a Raft implementation can pass a single `turmoil` seed by luck rather than correctness. The **Instructional Designer** named this as the top finding: three of the highest-stakes modules (03 leader election, 05 persistence, 06 log compaction) push "did it get lucky" questions onto Coachgremlin's *conceptual* tier, when the fix (running many seeds) is itself mechanical and belongs in the *deterministic* tier — exactly the anti-pattern Design Principle 2 exists to catch. The **Security-Conscious Reviewer** converged on the same root issue from a different angle: multi-seed testing is described as "likely" future practice, not a committed gate requirement.
2. **The "Confirmed, live, Spring 2026 offering" framing is stale given the doc's own dateline.** Both the **AI/ML Practitioner** and the **Technical Writer** independently flagged that a research pass dated 2026-08-29 cannot describe a Spring 2026 (Jan-May) course offering as currently "live."

## Single-persona findings (real, distinct mechanisms)

- **AI/ML Practitioner:** Raft paper **Figure 2** ("Summary of Raft") was mislabeled as "the state diagram" — that's **Figure 4** (the Follower/Candidate/Leader transition diagram). Also: the `madsim` description overstates it as generic `libc` interception; it actually works via crate substitution (`madsim-tokio`, patched `getrandom`, `cfg(madsim)`), not ambient interposition over arbitrary code.
- **Developer Evangelist:** the doc's strongest hook line ("a correct-looking Raft implementation can pass every unit test and still lose committed data...") is buried in paragraph three of an internal design doc instead of leading the pitch; the top-level `README.md` is still unrebranded scaffold boilerplate with a placeholder Quick Start; the 6.5840/openraft differentiation section leads with four hedges before the actual differentiators.
- **End-User/Learner:** Module 01's stated "no prerequisite" undersells that it's where the learner hand-builds the `turmoil`-backed RPC/fault-injection harness every later module depends on — the hardest onboarding ramp in the arc, framed as a warm-up. Also: no stated seed-count-per-submission policy, and no debugging aid at the Module 06→07 integration point (the first time persistence, compaction, and replication all combine).
- **Technical Writer:** ambiguous first-use of "the Gremlin" (Workshop Gremlin vs. Coachgremlin, both used elsewhere in the doc); `next-actions.md`'s "This Week" section breaks its own checkbox convention; `decisions.md`'s first ADR row uses "MIT 6.824/6.5840" with no explanation of the rename, resolved only later in `workshop-design.md`.
- **Skeptical Critic:** the Coachgremlin-solves-the-pass/fail-gap claim is listed alongside settled facts (Rust vs. Go, agent-native delivery) as though equally established, when Coachgremlin's conceptual grading for this workshop hasn't been built yet. Also: the hook ("not an opinion") sits in unacknowledged tension with the later admission that a `turmoil` seed can be wrong.
- **Instructional Designer:** only 3 of 9 modules (03, 05, 06) have a concretely named fault scenario in their gate description; the rest (01, 02, 04, 07, 08, 09) describe the gate generically. The capstone (09) has no stated distinguishing task — unlike `borrow-native`'s capstone (diagnose a seeded bug's root-cause concept), Heartbeat's 09 currently reads as "Module 08 with an extra label." Module 02's position before Module 03 is justified by a debugging-attribution argument, not a real dependency — already honestly disclosed in the doc, but dressed as sequence in the table.
- **Security-Conscious Reviewer:** no explicit statement that crash/restart exercises are confined to `turmoil`'s in-process simulation (as opposed to real processes/disk) — a plausible agent misinterpretation given the audience runs this through their own coding-agent harness. Also: no mention of whether agent-generated code auto-merges on a passing gate or requires human review first.

## Prioritized action list

Ordered by (a) cross-persona convergence, (b) severity.

1. **[Fixed this pass]** Fold multi-seed `turmoil` testing into the deterministic tier itself (a stated, published seed count/threshold), not a deferred Coachgremlin hope. *(Instructional Designer + Security-Conscious Reviewer)*
2. **[Fixed this pass]** Correct "Confirmed, live, Spring 2026 offering" to reflect it's the most recently published offering as of this research pass, not a claim about the current term. *(AI/ML Practitioner + Technical Writer)*
3. **[Fixed this pass]** Correct Figure 2 → Figure 4 for the state-transition diagram; correct the `madsim` mechanism description. *(AI/ML Practitioner)*
4. **[Fixed this pass]** Hedge the Coachgremlin-differentiator line as intent, not accomplished fact; cross-reference the hook line against the seed-count fix from #1 so the tension reads as resolved-by-design rather than quietly contradicted. *(Skeptical Critic)*
5. **[Fixed this pass]** Disambiguate "the Gremlin" on first use; fix `next-actions.md`'s checkbox inconsistency; clarify the 6.824→6.5840 rename on first mention in `decisions.md`. *(Technical Writer)*
6. **[Fixed this pass]** Name a concrete fault scenario for every module's gate, not just 03/05/06; give the capstone (09) a real distinguishing task modeled on `borrow-native`'s seeded-bug diagnosis. *(Instructional Designer)*
7. **[Fixed this pass]** Add an explicit line scoping crash/restart simulation to `turmoil`'s in-process simulation only. *(Security-Conscious Reviewer)*
8. **[Fixed this pass]** Flag Module 01's real difficulty (building the harness itself) explicitly rather than implying it's a warm-up. *(End-User/Learner)*
9. **[Deferred to Task 5 — Deliverables & branding]** Rewrite the top-level `README.md` with the buried hook line promoted to the opening, differentiators-before-hedges ordering, and a real Quick Start. *(Developer Evangelist)*
10. **[Deferred — no action needed now]** Module 06→07 integration debugging aid, and whether agent-generated code auto-merges vs. requires review — both real, but belong to content-building (Coachgremlin) and implementation-spec time respectively, not this design pass. Logged in `docs/next-actions.md`.

## Persona cost

Seven parallel general-purpose subagents, 15-105 seconds and ~31k-40k tokens each (widest variance from the Skeptical Critic and Technical Writer personas, both of which did extra file reads for cross-referencing).
