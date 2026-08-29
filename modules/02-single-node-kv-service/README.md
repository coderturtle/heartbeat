# Module 02: A Single-Node KV Service

## The question this module answers

What does a KV interface need to support before I even think about replicating it?

## Where it sits in the arc

Second module. Prerequisite: [Module 01](../01-rpc-over-unreliable-network/README.md) - needs the RPC layer to expose a service at all. Next: [Module 03, Raft: Leader Election](../03-raft-leader-election/README.md) - the hinge is that Module 07 later wraps *this exact interface* in Raft, so a bug found there is attributable to the Raft layer, not the API, only because this module's interface was proven correct on its own first. See [`modules/README.md`](../README.md) for the full arc and why this order (including the honest note that this ordering is a debugging-attribution argument, not a hard technical dependency).

## Learning objectives (placeholder - finalized when content is authored)

- Design a KV service interface (Get/Put/Append or equivalent) that's correct under concurrent client access, with no replication involved yet.
- Distinguish "this bug is in my KV logic" from "this bug is in my replication logic" - the exact distinction Module 07 will need this module's interface to have already settled.
- Use the Module 01 harness to inject latency/reordering into client requests and confirm the service's behavior doesn't depend on request ordering it can't guarantee.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

MIT 6.5840 Lab 2 (Key/Value Server) - deliberately unreplicated, the interface this whole arc eventually wraps in Raft. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor section for the corrected lab-order research behind this module's placement.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across both `turmoil` seed sets (practice and held-out, see `docs/workshop-design.md`) - concurrent client requests against the single-node service, under injected latency and reordering, never corrupt state or produce a result inconsistent with some valid serial order of the requests actually sent.
- **Conceptual tier (Coachgremlin):** confirms the interface design itself (not just this implementation) would still make sense once a caller can't assume a single in-process server - i.e., the API doesn't quietly bake in single-node assumptions Module 07 will have to unwind.

## Takeaway

An API-design checklist: what a service interface needs to support before it can survive being replicated (idempotency, request identification, no hidden ordering assumptions). Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's KV service passes the deterministic tier across both the practice and held-out seed sets, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
