# Module 08: Sharded Checkout Service

## The question this module answers

How do I split one `Checkout` service into many, without a resource's lock ever belonging to zero or two of them?

## Where it sits in the arc

Eighth module - last of the core arc before the capstone. Prerequisite: [Module 07](../07-fault-tolerant-checkout-service/README.md) - sharding assumes a working replicated `Checkout` service to shard in the first place. Next: [Module 09, Synthesis capstone](../09-synthesis-capstone/README.md). See [`modules/README.md`](../README.md) for the full arc and why this order.

## Learning objectives (placeholder - finalized when content is authored)

- Implement a shard controller that assigns resource namespaces (e.g., which repo, which branch) to replica groups and can rebalance that assignment as the fleet of agent sessions using `Checkout` grows.
- Route shard assignment using the *exact same* canonical resource identifier Module 02's exclusivity check uses (`docs/workshop-design.md`, fixed 2026-08-29 via doubt-driven-development) - a shard router that independently parses or normalizes the resource name (e.g., stripping a `refs/heads/` prefix for routing but not for the lock itself) can let two different-looking names for the same resource land on different shards and bypass the lock entirely.
- Implement shard migration such that a resource's lease state (holder, generation, expiry timestamp) and its `(client, resource)`-scoped dedup entries move with it - moving the lease but leaving generation or dedup state behind reopens the same stale-command and double-grant bugs this design already closed, just at the shard boundary instead of the replica boundary. The logical clock itself does *not* migrate (`docs/workshop-design.md`, corrected cycle 3 of this design's doubt-driven-development review: the clock belongs to the whole replica group, not to any one resource, so it can't transplant) - instead, the destination group's own clock must be confirmed to have already advanced past the migrated resource's last known timestamp before the migration counts as complete.
- Reason about the specific danger a single-node sharding scheme doesn't have: a partition during migration could leave a resource looking owned by two groups, or by none.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

MIT 6.5840 Lab 5 (Sharded KV), Parts A and B+C+D, as the *shape* to anchor to - `Checkout`'s own sharding key (resource namespace, not an arbitrary hash bucket) is this workshop's own design. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across both `turmoil` seed sets (practice and held-out, see `docs/workshop-design.md`), each simulating a shard migration in progress combined with a partition on the source or destination replica group - no seed in either set ever leaves a resource owned by zero or two groups at once.
- **Conceptual tier (Coachgremlin):** confirms the learner's migration protocol has an explicit, defensible answer for "who owns this resource right now" at every point during a handoff, not just at the start and end states.

## Takeaway

A shard-rebalancing/ownership-handoff checklist: the specific invariants to check whenever a system splits ownership of anything across multiple replicated groups. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's implementation passes the deterministic tier across both the practice and held-out seed sets, and Coachgremlin confirms the conceptual tier, per the gate above.

**Open question, not resolved by this design pass (see `docs/workshop-design.md`):** whether `turmoil` alone is sufficient to simulate this module's fault scenarios, or whether `madsim`'s stronger determinism guarantees are needed here specifically. Decided during content-building, not at skeleton stage.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
