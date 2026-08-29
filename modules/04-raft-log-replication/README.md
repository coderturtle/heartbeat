# Module 04: Raft: Log Replication

## The question this module answers

How does a leader get every follower's log to match its own, even a follower that's fallen behind?

## Where it sits in the arc

Fourth module. Prerequisite: [Module 03](../03-raft-leader-election/README.md) - a leader must exist before it can replicate a log to anyone. Next: [Module 05, Raft: Persistence](../05-raft-persistence/README.md) - the hinge is that persistence exists specifically to survive a crash mid-replication, so replication has to work before persisting it is meaningful. See [`modules/README.md`](../README.md) for the full arc and why this order.

**Still not `Checkout`-specific.** The log entries replicated here are opaque as far as this module is concerned - in Module 07 they'll carry `Checkout`'s own checkout/renew/return operations, but this module's log-matching property has to hold regardless of what the entries mean.

## Learning objectives (placeholder - finalized when content is authored)

- Implement AppendEntries-based log replication, including the consistency check that lets a leader detect where a follower's log diverges.
- Implement the log-matching property: if two logs contain an entry with the same index and term, all preceding entries are identical.
- Implement commit-index advancement correctly - a leader can only commit an entry from its current term by counting replicas, per the paper's §5.4.2 subtlety.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

Ongaro & Ousterhout, §5.3-5.4.1, Figure 2. MIT 6.5840 Lab 3B (Log Replication) for the reference test shape. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across both `turmoil` seed sets (practice and held-out, see `docs/workshop-design.md`), each simulating a follower that missed several entries during a partition - after the partition heals, that follower's log converges to exactly match the leader's, with the log-matching property verifiably intact, across every seed in both sets.
- **Conceptual tier (Coachgremlin):** confirms the learner can explain why an entry from a prior term needs indirect commitment (via a later entry in the current term) rather than being committed directly once replicated to a majority.

## Takeaway

A log-matching-property diagnostic checklist: how to tell, from a divergent log state, exactly where and why two replicas disagree. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's implementation passes the deterministic tier across both the practice and held-out seed sets, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
