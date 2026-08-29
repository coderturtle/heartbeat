# Module 06: Raft: Log Compaction & Snapshots

## The question this module answers

How do I discard old log entries without losing a lagging follower?

## Where it sits in the arc

Sixth module - last of the core Raft parts. Prerequisite: [Module 05](../05-raft-persistence/README.md) - you can only discard what's already durably persisted; compaction on top of an unreliable persistence layer would lose data outright. Next: [Module 07, Fault-Tolerant KV Service on Raft](../07-fault-tolerant-kv-service/README.md) - the hinge is that a complete Raft (all four parts) is what gets wrapped in a real service next. See [`modules/README.md`](../README.md) for the full arc and why this order.

## Learning objectives (placeholder - finalized when content is authored)

- Implement `Snapshot()`: discard log entries up to a given index once the state machine has applied them, without losing correctness for entries still needed.
- Implement `InstallSnapshot` RPC (Figure 13) for a follower lagging far enough behind that the leader no longer has the entries it needs.
- Reason about the boundary case: a follower that's *almost* caught up shouldn't need a snapshot at all - only one that's fallen behind the leader's compaction point should.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

Ongaro & Ousterhout, §7, Figure 13. MIT 6.5840 Lab 3D (Log Compaction) for the reference test shape. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across both `turmoil` seed sets (practice and held-out, see `docs/workshop-design.md`), each simulating a follower lagging far enough behind that its needed log entries were already compacted - that follower catches up via `InstallSnapshot`, not a replay of discarded entries, across every seed in both sets.
- **Conceptual tier (Coachgremlin):** confirms the snapshot boundary itself is chosen correctly (only servers that actually need it receive a snapshot) rather than over-broadly snapshotting servers that could have caught up via normal log replication.

## Takeaway

A snapshot-boundary decision guide: when a follower needs a snapshot versus normal replication, and how to verify the boundary is drawn correctly. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's implementation passes the deterministic tier across both the practice and held-out seed sets, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
