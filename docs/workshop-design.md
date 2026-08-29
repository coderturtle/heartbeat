# Workshop Design

> **Heartbeat.** Naming pass complete (see `docs/decisions.md`) — this doc was drafted after the name was chosen, so it uses the final name throughout.

## The one-line problem

Agent-literate practitioners who've already learned Rust the agent-native way (via `borrow-native` or equivalent) hit a second wall: distributed systems is a subject where almost nothing compiles wrong, yet almost everything can still be wrong — a correct-looking Raft implementation can pass every unit test and still lose committed data the first time a network partition heals badly. This workshop's bet is that the fix isn't more reading (MIT's own 6.5840 course material is excellent and freely available), it's a **deterministic gate that can actually see network faults**, not just type errors, paired with an agent-native harness and Coachgremlin's conceptual layer on top.

## Audience

Agent-literate practitioners: comfortable with git, the CLI, driving a coding agent daily, and already fluent in Rust specifically (ownership, borrowing, traits, `async`/await — the level `borrow-native` teaches to). **Not** a Rust-fundamentals workshop and **not** a true-beginner distributed-systems course — the one thing assumed unfamiliar is distributed systems itself: consensus, replication, partition tolerance, and what a network can actually do to a running protocol.

## Format

Self-paced, public repo. Matches `terminal-velocity` and `borrow-native`'s precedent.

## Subject vs. method (see `<hekton-machinery>/gremlins/workshop/workshop-gremlin.md`'s "Variant: Tech/Language Workshops")

- **Subject:** distributed systems — RPC over unreliable networks, single-node service design, leader election, log replication, persistence, log compaction, replicated state machines, and sharding.
- **Method:** agent-native — every module's core exercise runs through the learner's own coding-agent harness, graded first by a **deterministic check** (a real Raft implementation tested against real, simulated network faults), then by Coachgremlin's conceptual feedback on top.

The hook is the same shape as `borrow-native`'s, extended: "let a simulated, adversarial network — not an opinion — be the first gate." That claim leans on the multi-seed deterministic-tier design in "The deterministic gate, in the two-tier vocabulary" section below — a single arbitrary seed can get lucky the way a compiler never does, which is why that section makes "green across a published seed set" the standing requirement rather than "green once."

## Canonical-curriculum anchor (research pass, 2026-08-29)

Distributed systems has one dominant, free, current teaching anchor: **MIT's 6.5840 (formerly 6.824), "Distributed Systems."** Rather than invent a lab sequence from scratch, this workshop anchors to it directly — and one piece of that research corrected an assumption baked into this workshop's own initial scoping (see the callout below).

**Most recently published offering at the time of this research pass (2026-08-29)** — the Spring 2026 term ([pdos.csail.mit.edu/6.5840/](https://pdos.csail.mit.edu/6.5840/), schedule at [pdos.csail.mit.edu/6.5840/schedule.html](https://pdos.csail.mit.edu/6.5840/schedule.html)). By late August the Spring term itself has concluded; this is the most current published structure, not a claim that the course is in session right now:

| Lab | Title | Structure |
|---|---|---|
| 1 | MapReduce | single deliverable |
| 2 | Key/Value Server | single-node, **no replication yet** |
| 3 | Raft | Parts A (leader election), B (log replication), C (persistence), D (log compaction / snapshots) |
| 4 | KV Raft | Parts A, B+C — wraps Lab 2's KV interface in Lab 3's Raft |
| 5 | Sharded KV | Parts A, B+C+D |

**Correction, 2026-08-29:** this workshop was originally scoped, informally, with the arc "RPC → leader election → log replication → a KV store on top → sharding" — implying the KV interface comes *after* Raft is built. 6.5840's real current lab order says otherwise: **Lab 2 builds an unreplicated, single-node KV service before Raft exists at all** (Lab 3), and only then wraps that same KV interface in Raft (Lab 4). The pedagogical reason is straightforward once seen: it separates "design a correct KV API" from "make that API survive a partition," so a learner debugging Lab 4 already knows the KV interface itself is correct and can attribute every remaining bug to the replication layer. This workshop's module arc (below) follows the real order, not the original guess.

**`labrpc`, the course's own deterministic-gate harness** ([pdos.csail.mit.edu/6.824/labs/lab-raft1.html](https://pdos.csail.mit.edu/6.824/labs/lab-raft1.html)): a Go RPC package the test harness uses to delay, re-order, and discard RPCs to simulate network failures, plus (confirmed via Lab 3C/3D's persistence-across-reboot requirements) crash and restart individual servers mid-test. This is the exact role `turmoil` plays for this workshop, below — 6.5840 built its own because Go had no equivalent off the shelf in 2006; Rust does now.

**The Raft paper itself:** Ongaro & Ousterhout, "In Search of an Understandable Consensus Algorithm (Extended Version)" (2014), linked directly from the lab page as `raft-extended.pdf`. The lab's own instructions cite it by section: Figure 2 (the "Summary of Raft" — state variables and the RequestVote/AppendEntries RPC definitions every implementation is checked against), Figure 4 (the actual server-state transition diagram: Follower → Candidate → Leader), §5.2 (election timeouts), §5.4.1 (election restriction), §7 (log compaction), Figure 13 (the snapshot/InstallSnapshot RPC). This workshop uses the same paper as its own primary reading, module by module, rather than re-deriving Raft from a secondary source.

**Course-wide reading list** (from the schedule, for context on what this workshop deliberately does *not* cover): MapReduce (2004), GFS (2003), Paxos, Raft (2014), a linearizability paper, ZooKeeper (2010), Spanner (2012), Chain Replication (2004), FaRM (2015), IronFleet (2015), Memcached at Facebook (2013), AWS Lambda container loading (2023), Ray (2021), SUNDR (2004), Bitcoin (2008), Practical BFT (1999). This workshop is Raft-centric and deep (a deliberate scope decision, see below) — it does not attempt Paxos, BFT, or the systems-survey breadth of the full course.

**Differentiator against 6.5840 itself:** none of what follows is a claim that this workshop out-teaches a real MIT course taught by course staff with real grading infrastructure. Two differentiators are already true by construction: a **Rust implementation** (6.5840 is Go-only) and **agent-native delivery** (6.5840 assumes a human alone at a terminal). Two more are **stated intent, not yet built or evidenced** — flagged explicitly, the same honesty discipline `borrow-native` applied to its own teaching-method claims: **Coachgremlin's conceptual tier** is intended to catch what 6.5840's pass/fail tester structurally can't (a learner passing every test with a Raft implementation a real reviewer would reject), and a **keepable takeaway per module** is intended to leave the learner with an artifact, not just a grade — both are Coachgremlin's job at content-building time, not yet demonstrated. 6.5840's own labs and lecture notes remain the primary source of truth for the protocol itself; this workshop does not attempt to replace them.

## Existing Rust prior art (research pass, 2026-08-29)

- **`openraft`** ([github.com/databendlabs/openraft](https://github.com/databendlabs/openraft)) — a real, actively maintained, production-grade Rust Raft library, derived from the earlier `async-raft` project with a documented set of correctness fixes ([derived-from-async-raft.md](https://github.com/databendlabs/openraft/blob/main/derived-from-async-raft.md)). It's the consensus engine behind Databend's meta-service cluster: async/event-driven (no periodic ticks), generalized joint-consensus membership changes, fully pluggable storage/network. **This is a production library, not a teaching resource** — no tutorial arc, no graded exercises. It's this workshop's honest answer to "why not just use a library": because the point is understanding Raft well enough to *know* when `openraft` is the right call later, the same relationship `borrow-native` has to the Rust Book (a resource this workshop stands alongside, not against).
- No widely-known "6.824/6.5840 ported to Rust" community course surfaced in this research pass. Worth stating as "not found," not "doesn't exist" — a gap this workshop can genuinely fill if true, an honest unknown if it just wasn't surfaced.

## The deterministic gate: `turmoil`

**Dependency decision, human-confirmed 2026-08-29** (per this repo's `dependency_changes: human_required` governance gate): this workshop adopts **`turmoil`** ([docs.rs/turmoil](https://docs.rs/turmoil/latest/turmoil/)) as its Rust equivalent of `labrpc`, rather than hand-building a network-simulation harness from scratch.

- **What it does:** runs many simulated hosts deterministically on a single thread. One seeded RNG drives every scheduling and delivery decision, so a failing seed reproduces exactly. It supports dropping packets, partitioning hosts, injecting latency, reordering segments, and killing a connection mid-stream — a near-exact match for `labrpc`'s delay/reorder/discard, plus explicit partition support `labrpc` itself provides via its own `config.go` connect/disconnect calls.
- **Why not `madsim`** (the other real, current option — the "Magical Deterministic Simulator," used in production by RisingWave): `madsim` goes further, achieving determinism by substituting shim crates (`madsim-tokio`, a patched `getrandom` backend, etc. via `cfg(madsim)`) so `Instant::now()`/`getrandom` stay deterministic *for code built against those shims* — not by transparently intercepting arbitrary code the way a generic `libc` hook would. That's real integration cost (every dependency in the fault-injected path needs a madsim-aware build) this workshop doesn't need to pay yet — Raft's own timing (election timeouts, heartbeats) is expressible entirely through Tokio's mocked time, which `turmoil` already covers without any crate substitution. Escalating to `madsim` stays an option if a later module (sharding, most likely) hits a case `turmoil` can't simulate.
- **Con of not adopting either:** hand-rolling this harness means its own bugs are indistinguishable from a learner's Raft bugs when a test fails mysteriously — exactly the failure mode a deterministic gate exists to prevent. Building one from scratch also means re-solving a problem `turmoil` already solves, for a piece of infrastructure that isn't this workshop's actual teaching content.

## The module arc

Anchored to 6.5840's real lab order (corrected above), scoped for a learner who already knows Rust — modules that would be Rust-fundamentals review for this audience are skipped entirely; every module here is genuinely new (distributed-systems) content. Each module names its **hard prerequisite** explicitly, per the Workshop Gremlin's concept-dependency-arc requirement.

| # | Module | Hard prerequisite | 6.5840 anchor | Gate's named fault scenario (deterministic tier) |
|---|---|---|---|---|
| 01 | RPC Over an Unreliable Network | none (Rust fluency assumed) — **but see the callout below: this is not a warm-up** | "RPC and Threads" lecture; `labrpc`'s own role, reimplemented as this workshop's `turmoil`-backed harness | The harness itself, under test: dropped/delayed/reordered messages arrive at a toy RPC service exactly as configured, across a fixed seed set |
| 02 | A Single-Node KV Service | 01 (needs the RPC layer to expose a service at all) | Lab 2 — deliberately unreplicated; the interface this whole arc eventually wraps in Raft | Concurrent client requests against the single-node service under injected latency/reordering never corrupt state, even with no replication involved yet |
| 03 | Raft: Leader Election | 01 (RPCs) | Lab 3A; Raft paper §5.2, Fig. 2, Fig. 4 | A `turmoil` partition isolating the current leader; exactly one new leader is elected in the majority partition, never two leaders in the same term |
| 04 | Raft: Log Replication | 03 (a leader must exist before it can replicate a log) | Lab 3B; Raft paper §5.3-5.4.1 | A follower that missed several entries catches up correctly after a simulated partition heals, with the log-matching property intact |
| 05 | Raft: Persistence | 04 (persisted state is the log/term/vote Module 04 already produces) | Lab 3C; Raft paper Fig. 2's persistent-state fields | A `turmoil`-simulated crash-and-restart mid-replication recovers exactly the pre-crash persisted state, never re-votes in an already-decided term |
| 06 | Raft: Log Compaction & Snapshots | 05 (you can only discard what's already durably persisted) | Lab 3D; Raft paper §7, Fig. 13 | A follower lagging far enough behind that its needed log entries were already compacted catches up via `InstallSnapshot`, not a replay of discarded entries |
| 07 | Fault-Tolerant KV Service on Raft | 02 (the KV interface) and 06 (a complete Raft) | Lab 4 | The KV service stays linearizable across a `turmoil`-simulated leader failover mid-request — a client never observes a committed write disappear or an uncommitted one become visible |
| 08 | Sharded KV Service | 07 | Lab 5 | A shard migration in progress, combined with a simulated partition on the source or destination group, never leaves a key owned by zero or two groups at once |
| 09 | Synthesis capstone | all of the above | — | A real, seeded bug in the accumulated project spanning 3+ concepts from the arc — the learner must get every module's `turmoil` suite green again (deterministic tier) *and* correctly diagnose, in writing, which arc concept was the actual root cause before fixing it (conceptual tier), mirroring `borrow-native`'s own capstone shape |

**Module 01 is not a warm-up.** Flagged directly per this workshop's own Review Panel (`docs/review-panel/2026-08-29-initial-design.md`, End-User/Learner persona): "no prerequisite" describes the *concept* dependency, not the *engineering* difficulty. Module 01 is where the learner builds the `turmoil`-backed harness every later module's gate depends on, with zero distributed-systems intuition yet — closer to this workshop's hardest onboarding ramp than a gentle first step. Content-building should treat it accordingly (more scaffolding, not less).

### Why this order

Modules 03-06 are 6.5840's own Raft parts kept in their original order rather than re-split — Ongaro & Ousterhout's paper and the lab's own structure already argue for leader election before replication (nothing to replicate without a leader), replication before persistence (persistence exists to survive a crash mid-replication), and persistence before compaction (compaction discards persisted state, so the thing being discarded has to be reliably saved first). Module 02 sits before Module 03, matching 6.5840's real order and this design doc's own correction above: building the KV interface without replication first means every bug surfaced in Module 07 is attributable to the Raft layer, not the API design — worth being honest that this is a debugging-attribution argument, not a hard technical dependency the way 03→04→05→06 are (Module 03 doesn't actually require Module 02's code to exist). Module 08 (sharding) is last among the core modules because it assumes a working replicated KV service to shard in the first place.

## The deterministic gate, in the two-tier vocabulary `borrow-native` established

1. **Deterministic tier (primary).** `cargo test` green against a `turmoil`-simulated network, **run across a fixed, published set of seeds (not one)** — including the specific fault the module is about (a partition during leader election for Module 03, a crash-and-restart for Module 05, a lagging follower needing a snapshot for Module 06, etc.). The exact seed count is a content-building decision (Coachgremlin's job, calibrated per module against how narrow that module's race window actually is), but "green on every seed in the published set," not "green on the one seed the exercise ships with," is the standing requirement from this design pass forward. **Revised 2026-08-29, from this workshop's own Review Panel** (`docs/review-panel/2026-08-29-initial-design.md`, Instructional Designer + Security-Conscious Reviewer): "did it get lucky with the test's specific timing" is a mechanically-answerable question — more seeds either does or doesn't expose the bug — so it belongs in this tier, not pushed onto Coachgremlin's judgment as a conceptual question. All crash/restart and partition simulation is confined to `turmoil`'s in-process, single-thread simulation — no exercise in this arc kills or restarts a real process, or persists state to a real (non-sandboxed) disk path, to test fault tolerance.
2. **Conceptual tier (secondary, Coachgremlin).** Idiom and, more specifically to this subject, protocol correctness a passing multi-seed test suite still can't fully guarantee: is the persisted state actually the minimal correct set (Fig. 2), or over-broad in a way a reviewer would flag even though it happens to survive every published seed? Did the learner reason about *why* the protocol is safe (can explain the invariant in writing), or pattern-match a fix that merely stopped the specific failures they saw?

**Open, stated honestly:** even a large published seed set is finite — an adversarially-unlucky attempt could still pass by chance, just at much lower odds than passing a single arbitrary seed. `borrow-native`'s own module dry runs found real cases where a deterministic tier alone couldn't distinguish a correct attempt from a naive one; this workshop's Coachgremlin dry runs (content-building phase, not this design pass) need to test whether the multi-seed requirement above closes that gap enough in practice, or whether some modules need a still-stronger deterministic check (e.g., property-based seed generation rather than a fixed set).

## What you keep

Per the Workshop Gremlin's takeaway requirement — concrete takeaways are Coachgremlin's job at content-building time, but the intended *shape* per module:

| # | Module | Intended takeaway shape |
|---|---|---|
| 01 | RPC Over an Unreliable Network | A `turmoil`-based network-fault-injection harness template, reusable on future async Rust projects |
| 02 | A Single-Node KV Service | An API-design checklist for "what does this interface need to support before I even think about replicating it" |
| 03 | Raft: Leader Election | A leader-election diagnostic playbook (term/vote/timeout reasoning) |
| 04 | Raft: Log Replication | A log-matching-property diagnostic checklist |
| 05 | Raft: Persistence | A "what actually needs to survive a crash" checklist, generalizable beyond Raft |
| 06 | Raft: Log Compaction & Snapshots | A snapshot-boundary decision guide |
| 07 | Fault-Tolerant KV Service on Raft | A layering playbook: how to keep a service's API and its consensus layer independently testable |
| 08 | Sharded KV Service | A shard-rebalancing/ownership-handoff checklist |
| 09 | Synthesis capstone | A personal distributed-systems diagnostic playbook compressing the whole arc |

## Build-in-public build log

Published as a dated build-log/journal via GitHub Pages, reusing the Astro-on-Pages pipeline `terminal-velocity` built and `borrow-native` reused — see that pipeline's implementation notes in `<hekton-machinery>/gremlins/workshop/workshop-gremlin.md`'s Build-log/Pages publisher agent section.

## What's explicitly out of scope for this design pass

- All real module content (fixtures, exercises, rubrics) — Coachgremlin's job, run later, one module at a time.
- The actual Astro site content and first Pages deploy.
- Deciding whether `madsim` is ever needed for Module 08 specifically — flagged above as a live open question, not resolved here.
- Any claim that `turmoil`-based tests catch every real Raft bug a given module's exercise could contain — an explicitly open risk, not a settled finding, per the two-tier gate section above.
