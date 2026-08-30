# Module 05: Raft: Persistence

## The question this module answers

What has to survive a crash, and what happens if it doesn't?

## Where it sits in the arc

Fifth module. Prerequisite: [Module 04](../04-raft-log-replication/README.md) - the persisted state is exactly the log/term/vote data Module 04 already produces; persistence has nothing to save until replication exists. Next: [Module 06, Raft: Log Compaction & Snapshots](../06-raft-log-compaction-snapshots/README.md) - the hinge is that compaction discards persisted state, so the thing being discarded has to be reliably saved first. See [`modules/README.md`](../README.md) for the full arc and why this order.

**Still not `Checkout`-specific.** What's persisted here (currentTerm, votedFor, log) is Raft's own state, not `Checkout`'s leases - once Module 07 wraps `Checkout` in this engine, a lease's durability rides entirely on this module's correctness, without this module needing to know that.

## Learning objectives (placeholder - finalized when content is authored)

- Identify the minimal correct set of state Raft must persist before responding to an RPC (Figure 2's persistent-state fields: currentTerm, votedFor, log).
- Implement save-on-every-relevant-change, not save-on-a-timer - a subtle correctness requirement, not just a performance one.
- Reason about what happens if a server crashes and restarts mid-election or mid-replication, and why re-reading persisted state must never contradict a promise already made before the crash.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

Ongaro & Ousterhout, Figure 2's persistent-state fields. MIT 6.5840 Lab 3C (Persistence) for the reference test shape. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across the practice set and the grading draw (see `docs/workshop-design.md`), each simulating a crash-and-restart of one or more servers mid-replication - every restarted server recovers exactly its pre-crash persisted state and never re-votes in a term it already decided, across every drawn seed. All crash/restart simulation is confined to `turmoil`'s in-process simulation, never a real process or real disk path.
- **Conceptual tier (Coachgremlin):** confirms the persisted state is the *minimal* correct set (Figure 2), not over-broad in a way that happens to survive every practice seed but would break under a fault the seed set didn't happen to hit.

## Takeaway

A "what actually needs to survive a crash" checklist - generalizable well beyond Raft, to any stateful service reasoning about durability. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's implementation passes the deterministic tier across the practice set and the grading draw, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
