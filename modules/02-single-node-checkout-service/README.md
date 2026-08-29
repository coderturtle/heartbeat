# Module 02: A Single-Node Checkout Service

## The question this module answers

What does an exclusive-ownership API need to guarantee before I even think about replicating it?

## Where it sits in the arc

Second module. Prerequisite: [Module 01](../01-rpc-over-unreliable-network/README.md) - needs the RPC layer to expose a service at all. Next: [Module 03, Raft: Leader Election](../03-raft-leader-election/README.md) - the hinge is that Module 07 later wraps *this exact `Checkout` interface* in Raft, so a bug found there is attributable to the Raft layer, not the API, only because this module's interface was proven correct on its own first. See [`modules/README.md`](../README.md) for the full arc, why this order (including the honest note that this ordering is a debugging-attribution argument, not a hard technical dependency), and how `Checkout` is the one product this whole workshop builds.

## Learning objectives (placeholder - finalized when content is authored)

- Design `Checkout`'s core API - `Checkout(resource, holder, lease_duration)`, `Renew(resource, holder)`, `Return(resource, holder)`, `Status(resource)` - correct under concurrent client access, with no replication involved yet.
- Get lease expiry right: an unrenewed lease must become available again on its own, so a crashed holder never locks a resource forever.
- Distinguish "this bug is in my checkout logic" from "this bug is in my replication logic" - the exact distinction Module 07 will need this module's interface to have already settled.
- Use the Module 01 harness to inject latency/reordering into client requests and confirm the service's behavior doesn't depend on request ordering it can't guarantee (a `Renew` racing a `Return` for the same resource is the obvious case to get wrong).

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

MIT 6.5840 Lab 2 (Key/Value Server) as the *shape* to anchor to - deliberately unreplicated, the pattern this workshop's own `Checkout` interface follows before Module 07 wraps it in Raft. `Checkout`'s actual semantics (exclusive, leased ownership rather than a bare key-value `Get`/`Put`) are this workshop's own design, not lifted from the lab. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section and "The shared project: `Checkout`" section.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across both `turmoil` seed sets (practice and held-out, see `docs/workshop-design.md`) - concurrent checkout requests against the single-node service, under injected latency and reordering, never grant the same resource to two holders at once, and every lease correctly expires and becomes available again, across every seed in both sets.
- **Conceptual tier (Coachgremlin):** confirms the interface design itself (not just this implementation) would still make sense once a caller can't assume a single in-process server - i.e., the API doesn't quietly bake in single-node assumptions Module 07 will have to unwind, and lease-expiry timing doesn't secretly depend on wall-clock behavior this service won't have once it's replicated.

## Takeaway

An exclusive-ownership API checklist: idempotent checkout requests, renewal-vs-return races, lease-duration tradeoffs, and what an interface needs to survive being replicated. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's `Checkout` service passes the deterministic tier across both the practice and held-out seed sets, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
