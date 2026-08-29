# Module 03: Raft: Leader Election

## The question this module answers

How does a cluster agree on exactly one leader, even when the network actively works against it?

## Where it sits in the arc

Third module. Prerequisite: [Module 01](../01-rpc-over-unreliable-network/README.md) (RPCs) - Raft's leader election is built entirely on RequestVote/AppendEntries RPCs over the harness Module 01 produced. Next: [Module 04, Raft: Log Replication](../04-raft-log-replication/README.md) - the hinge is that there's nothing to replicate until a leader exists. See [`modules/README.md`](../README.md) for the full arc and why this order.

## Learning objectives (placeholder - finalized when content is authored)

- Implement Raft's leader-election state machine (Follower/Candidate/Leader) per the Raft paper's Figure 2 (state summary) and Figure 4 (the transition diagram).
- Implement the election-restriction rule (§5.4.1): a candidate can't win without a log at least as up-to-date as a majority of the cluster.
- Reason about split votes and election timeouts under real network delay, not just in the no-fault case.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

Ongaro & Ousterhout, "In Search of an Understandable Consensus Algorithm (Extended Version)," §5.2 and §5.4.1, Figures 2 and 4. MIT 6.5840 Lab 3A (Leader Election) for the reference test shape. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across a published set of `turmoil` seeds, each simulating a partition that isolates the current leader - exactly one new leader is elected in the resulting majority partition, and never two leaders exist in the same term across any seed in the set.
- **Conceptual tier (Coachgremlin):** confirms the learner can explain, in writing, why the election-restriction rule prevents a stale candidate from winning, rather than having implemented it by pattern-matching the paper's pseudocode without understanding why it's there.

## Takeaway

A leader-election diagnostic playbook: how to reason about term numbers, votes, and timeouts when an election isn't converging. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's implementation passes the deterministic tier across the full published seed set, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
