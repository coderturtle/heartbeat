# Module 07: Fault-Tolerant Checkout Service on Raft

## The question this module answers

How do I wrap a working checkout/lease API in Raft without either layer leaking its bugs into the other?

## Where it sits in the arc

Seventh module - the first real integration point in the arc, and the first module where `Checkout`'s own product identity (from Module 02) meets the generic Raft engine (Modules 03-06). Prerequisites: [Module 02](../02-single-node-checkout-service/README.md) (the `Checkout` interface) and [Module 06](../06-raft-log-compaction-snapshots/README.md) (a complete Raft). This is the highest-risk step in the whole workshop, per this workshop's own Review Panel (`docs/review-panel/2026-08-29-initial-design.md`, End-User/Learner persona): it's the first time persistence, compaction, and replication all combine at once, and unlike Modules 03-06 (one new concept each), a bug here could originate in any layer built so far. Next: [Module 08, Sharded Checkout Service](../08-sharded-checkout-service/README.md) - the hinge is that sharding assumes a working replicated `Checkout` service to shard in the first place. See [`modules/README.md`](../README.md) for the full arc and why this order.

## Learning objectives (placeholder - finalized when content is authored)

- Layer `Checkout`'s request handling on top of a Raft instance: propose each checkout/renew/return as a log entry, apply committed entries to the lease-table state machine in order. On a retry, reuse the exact same `(client_id, resource, sequence number)` tuple from Module 02's own dedup mechanism rather than minting a new identifier - not a separately-named "request ID," the same tuple, same term (`docs/workshop-design.md`, corrected cycle 3 of this design's doubt-driven-development review after an earlier draft used two different names for the same thing).
- Handle client retries and duplicate requests correctly across a leader failover, using the dedup and lease-generation fencing Module 02 already built - a `Checkout` call submitted to a leader that then loses leadership must not be silently lost (the caller thinks it failed, but the lease was actually granted) or double-applied (two holders end up believing they own the same resource). A deduplicated response replays the exact original outcome, including denials - not just successes.
- Decide expiry from replicated state alone, never from any single replica's own clock read, and keep that replicated clock advancing even when no client is calling in (a resource nobody touches must still eventually be observed to expire). A newly-elected leader must never propose a timestamp lower than the highest one already committed - a real risk specifically after a leadership change or a partition healing.
- Serve `Status` (and checkout-denial responses) linearizably using read-index or routing through the log - never a follower's local state, and never leader-lease-read, which depends on bounded real-clock skew and would reintroduce the exact wall-clock dependency this module's expiry design just eliminated.
- Diagnose, when a test fails, whether the bug is in `Checkout`'s own logic, the Raft layer, or the boundary between them - the exact skill Module 02's interface design was meant to make possible.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

MIT 6.5840 Lab 4 (KV Raft), Parts A and B+C, as the *shape* to anchor to - `Checkout`'s actual semantics are this workshop's own design, layered onto that shape rather than lifted from it. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section and "The shared project: `Checkout`" section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across the practice set and the grading draw (see `docs/workshop-design.md`), each simulating a leader failover mid-request - the `Checkout` service stays linearizable across every seed: no client ever observes a granted lease disappear, nor a denied checkout silently become granted.
- **Conceptual tier (Coachgremlin):** confirms the learner can correctly attribute a failing test to `Checkout`'s own logic, the Raft layer, or the integration boundary, and explain the reasoning - not just that they eventually found the bug by trial and error.

## Takeaway

A layering playbook: how to keep a service's API and its consensus layer independently testable, and how to localize a bug to one or the other. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's implementation passes the deterministic tier across the practice set and the grading draw, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
