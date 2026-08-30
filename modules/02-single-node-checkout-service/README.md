# Module 02: A Single-Node Checkout Service

## The question this module answers

What does an exclusive-ownership API need to guarantee before I even think about replicating it?

## Where it sits in the arc

Second module. Prerequisite: [Module 01](../01-rpc-over-unreliable-network/README.md) - needs the RPC layer to expose a service at all. Next: [Module 03, Raft: Leader Election](../03-raft-leader-election/README.md) - the hinge is that Module 07 later wraps *this exact `Checkout` interface* in Raft, so a bug found there is attributable to the Raft layer, not the API, only because this module's interface was proven correct on its own first. See [`modules/README.md`](../README.md) for the full arc, why this order (including the honest note that this ordering is a debugging-attribution argument, not a hard technical dependency), and how `Checkout` is the one product this whole workshop builds.

## Learning objectives (placeholder - finalized when content is authored)

- Design `Checkout`'s core API - `Checkout(resource, holder, lease_duration) -> generation`, `Renew(resource, holder, generation)`, `Return(resource, holder, generation)`, `Status(resource)` - correct under concurrent client access, with no replication involved yet. `lease_duration` is client-requested but server-capped to a stated maximum (`docs/workshop-design.md`, added cycle 2 of this design's doubt-driven-development review) - an uncapped initial grant reopens the same unbounded-hold abuse the generation/lease-expiry mechanism exists to prevent.
- Get lease expiry right by proposing a clock advancement on your own, not just when a client calls in - an idle resource nobody touches must still eventually be observed to expire (`docs/workshop-design.md`, added cycle 2: a clock that only advances on client traffic never notices an untouched lease has expired). No real wall-clock reliance from the start, since Module 07 makes that a hard replication requirement later.
- Key every resource by a canonical, opaque identifier, canonicalized once at the API boundary before any other logic runs - no implicit namespace parsing separate from the exclusivity check - so `main` and `refs/heads/main` can't alias past the lock later, once Module 08 shards by namespace.
- Give every lease a generation number that `Checkout` returns to the caller and increments on each new grant; reject `Renew`/`Return` calls whose supplied generation doesn't match the resource's current one - the fencing mechanism that stops a delayed command from affecting a lease granted after the one it targeted. A resource's generation counter persists even while it has no active lease - don't reset or drop it just because nothing currently holds the resource, or a later `Checkout` could reissue an already-used generation.
- Cache and replay the exact outcome (granted or denied, generation-stamped) for a repeated request, keyed by `(client_id, resource, sequence number)` - scoped per resource, not just per client (`docs/workshop-design.md`, corrected cycle 3 of this design's doubt-driven-development review: a client holding two independent leases and renewing both concurrently needs two independent sequence counters, or the second renewal's cache entry silently clobbers the first's).
- Distinguish "this bug is in my checkout logic" from "this bug is in my replication logic" - the exact distinction Module 07 will need this module's interface to have already settled.
- Use the Module 01 harness to inject latency/reordering into client requests and confirm the service's behavior doesn't depend on request ordering it can't guarantee (a `Renew` racing a `Return` for the same resource is the obvious case to get wrong).

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

MIT 6.5840 Lab 2 (Key/Value Server) as the *shape* to anchor to - deliberately unreplicated, the pattern this workshop's own `Checkout` interface follows before Module 07 wraps it in Raft. `Checkout`'s actual semantics (exclusive, leased ownership rather than a bare key-value `Get`/`Put`) are this workshop's own design, not lifted from the lab. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section and "The shared project: `Checkout`" section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across the practice set and the grading draw (see `docs/workshop-design.md`) - concurrent checkout requests against the single-node service, under injected latency and reordering, never grant the same resource to two holders at once, and every lease correctly expires and becomes available again, across every drawn seed.
- **Conceptual tier (Coachgremlin):** confirms the interface design itself (not just this implementation) would still make sense once a caller can't assume a single in-process server - i.e., the API doesn't quietly bake in single-node assumptions Module 07 will have to unwind, and lease-expiry timing doesn't secretly depend on wall-clock behavior this service won't have once it's replicated.

## Takeaway

An exclusive-ownership API checklist: idempotent checkout requests, renewal-vs-return races, lease-duration tradeoffs, and what an interface needs to survive being replicated. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's `Checkout` service passes the deterministic tier across the practice set and the grading draw, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
