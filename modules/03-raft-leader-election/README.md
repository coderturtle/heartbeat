# Module 03: Raft: Leader Election

## The question this module answers

How does a cluster agree on exactly one leader, even when the network actively works against it?

## Where it sits in the arc

Third module. Prerequisite: [Module 01](../01-rpc-over-unreliable-network/README.md) (RPCs) - Raft's leader election is built entirely on RequestVote/AppendEntries RPCs over the harness Module 01 produced, and reuses its wire framing directly (`crate::rpc::{read_framed, write_framed}`). Next: [Module 04, Raft: Log Replication](../04-raft-log-replication/README.md) - the hinge is that there's nothing to replicate until a leader exists. See [`modules/README.md`](../README.md) for the full arc and why this order.

**Not `Checkout`-specific, deliberately.** This module (and 04-06) build the generic Raft engine `Checkout` will run on starting Module 07 - leader election doesn't know or care that it'll eventually carry lease-and-holder state. See [`modules/README.md`](../README.md)'s "One shared project" section for why this arc is honestly uneven rather than adding a `Checkout` feature every module.

## Exercise: implement `RaftNode`

Runs against `fixtures/checkout/src/raft/`. `types.rs`, `timer.rs`, `connector.rs`, and `transport.rs` are provided infrastructure - wire message types, a compaction-ready log abstraction (unused by this module's own logic, shipped now so Module 06 needs no rework), deterministic election-timeout jitter, and wire framing/RPC exchange reusing Module 01's own `read_framed`/`write_framed`. Your exercise is entirely in `node.rs`, currently four `todo!()`s on `RaftNode<C: Connector>`: a two-phase constructor (`new`, then `start`), and `handle_request_vote`/`handle_append_entries`.

> Implement the Figure 2 leader-election algorithm: election timeouts, requesting and granting votes, becoming leader on a majority, and stepping down on any higher term seen anywhere. **You choose your own interior-mutability/synchronization strategy** - a `Mutex`, an `RwLock`, a channel-fed actor task - a deliberate, human-confirmed design decision matching MIT 6.5840 Lab 3A's own philosophy: the concurrency design is part of the exercise, not scaffolded away, the same way it's part of what real Raft implementations have to get right. `node.rs`'s own doc comments name several specific traps this way of testing surfaces (holding your lock across an outbound call, an un-`biased` `tokio::select!`, `HashMap`-ordered iteration) - read them before you start, not after you're stuck. Get `cargo test --test module_03_raft_election` green and `cargo clippy --tests -- -D warnings` clean, then check your diff against the rubric below.

## Rubric

See [`rubric.md`](rubric.md) for the full table and rationale.

**Before trusting a green `cargo test` alone:** this module's own dry run and two rounds of doubt-driven-development (`runs/2026-08-31-module-03-dry-run/`) found several real gaps that a passing test suite alone would have hidden. Two are worth knowing about directly, since they shape what "green" actually means here: (1) the partition test's *liveness* check originally couldn't fail no matter what a learner's implementation did after a partition - fixed to require a genuinely new leader, in a higher term, on a different node; (2) nothing in the original suite tested that election-timeout jitter actually re-randomizes on every attempt, against a provided RNG field shaped in a way that makes forgetting to do so the easy mistake to make by accident. Both are fixed now, but the underlying lesson generalizes: a green `cargo test` here means the *current* suite didn't catch a problem, not that no problem exists - the same discipline Module 01's own dry run established for `cargo clippy` vs. `cargo test` catching different bug classes.

## Required gate

- **Deterministic tier:** `cargo test --test module_03_raft_election` green (6 tests: single-node self-election, a no-fault 3-node cluster, election under injected latency meaningfully above `turmoil`'s own ambient default, liveness after the current leader is isolated by a real partition, a dedicated 30-seed split-vote stress test, and same-seed-twice determinism across the full seed set) and `cargo clippy --tests -- -D warnings` clean. `SEEDS = [1, 2, 3, 5, 8]` (and `SPLIT_VOTE_SEEDS = 100..130` for the split-vote test specifically) are a small, published, illustrative set for this stage - not the real practice/held-out split `docs/workshop-design.md`'s deterministic-gate section describes; passing across this set is not itself a grading criterion.
- **Conceptual tier (Coachgremlin):** confirms the learner can explain, in writing, why the election-restriction rule (§5.4.1: a candidate can't win without a log at least as up-to-date as a majority) matters, even though this module's own test suite can't independently exercise it - every node's log is empty throughout Module 03 (nothing populates it until Module 04's log replication), so every candidate's `last_log_index`/`last_log_term` ties at `(0, 0)` in every scenario this suite runs. The check must still be implemented correctly (a `RequestVote` handler that ignores these fields entirely is a real, if currently untested, correctness gap Module 04's own test suite will very likely surface once logs actually diverge) - the conceptual tier is how this module confirms the learner understands *why* the check exists, since the deterministic tier can't yet confirm they built it.

## Takeaway

A leader-election diagnostic playbook: how to reason about term numbers, votes, and timeouts when an election isn't converging, plus two reusable test-design patterns from this module's own dry run - an independent, harness-side wire-observation cross-check that doesn't trust a learner's own self-reported state, and a dedicated large-seed-set stress test for probabilistically forcing a split vote when the property under test (re-randomization) can't be verified directly from a black-box event log.

## Stop condition

The learner's `RaftNode` passes the deterministic tier (`cargo test --test module_03_raft_election` green, `cargo clippy --tests -- -D warnings` clean) and Coachgremlin confirms the conceptual tier, per `rubric.md`.

## Learning objectives

- Implement Raft's leader-election state machine (Follower/Candidate/Leader) per the Raft paper's Figure 2 (state summary) and Figure 4 (the transition diagram).
- Understand the election-restriction rule (§5.4.1) and why it prevents a stale candidate from winning, even though this module's own exercise can't independently test it (see the Conceptual tier above) - a real instance of "understand why, because the deterministic gate can't check it yet," not a gap glossed over.
- Design your own concurrency strategy for a Raft node under real concurrent access (RPC handlers, an election timer, and your own outbound calls all touching the same state) - matching MIT 6.5840 Lab 3A's own philosophy rather than a provided scaffold, deliberately, per a human-confirmed design decision made before this module's fixture was authored.
- Reason about split votes and election timeouts under real network delay and a real partition, not just the no-fault case - including why re-randomizing the election timeout on every attempt (not just once, at startup) is what actually keeps a split vote from repeating forever.

## Exercise material to draw from (not a spec)

Ongaro & Ousterhout, "In Search of an Understandable Consensus Algorithm (Extended Version)," §5.2 and §5.4.1, Figures 2 and 4. MIT 6.5840 Lab 3A (Leader Election) for the reference test shape. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section.

---

> **Real, not a placeholder.** Exercise, fixture, test suite, and rubric all exist. The fixture API split went through three doubt-driven-development cycles before any code was written; the test suite was dry-run against a correct attempt and a real naive attempt, then itself went through a doubt-driven-development cycle (`runs/2026-08-31-module-03-dry-run/`). See [`modules/README.md`](../README.md) for workshop-wide status - Modules 04-09 are still skeleton only.
