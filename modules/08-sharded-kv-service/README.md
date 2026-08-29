# Module 08: Sharded KV Service

## The question this module answers

How do I split one replicated service into many, without a key ever belonging to zero or two of them?

## Where it sits in the arc

Eighth module - last of the core arc before the capstone. Prerequisite: [Module 07](../07-fault-tolerant-kv-service/README.md) - sharding assumes a working replicated KV service to shard in the first place. Next: [Module 09, Synthesis capstone](../09-synthesis-capstone/README.md). See [`modules/README.md`](../README.md) for the full arc and why this order.

## Learning objectives (placeholder - finalized when content is authored)

- Implement a shard controller that assigns key ranges (or hash buckets) to replica groups and can rebalance that assignment.
- Implement shard migration between replica groups such that a key is served correctly during the handoff, not just before and after it.
- Reason about the specific danger a single-node sharding scheme doesn't have: a partition during migration could leave a key looking owned by two groups, or by none.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

MIT 6.5840 Lab 5 (Sharded KV), Parts A and B+C+D. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across a published set of `turmoil` seeds, each simulating a shard migration in progress combined with a partition on the source or destination replica group - no seed in the set ever leaves a key owned by zero or two groups at once.
- **Conceptual tier (Coachgremlin):** confirms the learner's migration protocol has an explicit, defensible answer for "who owns this key right now" at every point during a handoff, not just at the start and end states.

## Takeaway

A shard-rebalancing/ownership-handoff checklist: the specific invariants to check whenever a system splits ownership of anything across multiple replicated groups. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's implementation passes the deterministic tier across the full published seed set, and Coachgremlin confirms the conceptual tier, per the gate above.

**Open question, not resolved by this design pass (see `docs/workshop-design.md`):** whether `turmoil` alone is sufficient to simulate this module's fault scenarios, or whether `madsim`'s stronger determinism guarantees are needed here specifically. Decided during content-building, not at skeleton stage.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
