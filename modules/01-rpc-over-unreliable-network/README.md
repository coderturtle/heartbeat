# Module 01: RPC Over an Unreliable Network

## The question this module answers

How do I build an RPC layer, and the harness that can lie to it convincingly?

## Where it sits in the arc

First module. No conceptual prerequisite - but see the callout below, this is not a warm-up. Next: [Module 02, A Single-Node KV Service](../02-single-node-kv-service/README.md), which builds its first real service on top of the RPC layer this module produces. See [`modules/README.md`](../README.md) for the full arc and why this order.

**This module is not a gentle warm-up.** It's where you build the `turmoil`-backed network-fault-injection harness every later module's deterministic gate depends on - real engineering work, with zero distributed-systems intuition yet to lean on. See `docs/workshop-design.md`'s callout on this, added after this workshop's own Review Panel flagged the "no prerequisite" framing as underselling the difficulty.

## Learning objectives (placeholder - finalized when content is authored)

- Build an async Rust RPC mechanism (request/response over a simulated network) from first principles, without reaching for an existing RPC framework.
- Use `turmoil` to inject latency, packet loss, reordering, and partitions into that RPC mechanism, and observe each fault's effect directly.
- Explain, from having built it, what MIT 6.5840's own `labrpc` package does for Go learners and why an equivalent doesn't yet exist off the shelf for Rust.

## Exercise material to draw from (not a spec - Coachgremlin authors the real exercise later)

MIT 6.5840's "RPC and Threads" lecture and the `labrpc` package's role in the course (`pdos.csail.mit.edu/6.824/labs/lab-raft1.html` describes its use in the Raft lab specifically). `turmoil`'s own documentation (`docs.rs/turmoil`) for the Rust-side simulation primitives. See [`docs/workshop-design.md`](../../docs/workshop-design.md)'s curriculum-anchor and gate-harness sections.

## Required gate (placeholder - shape decided now, real rubric written later)

- **Deterministic tier:** `cargo test` green across a published set of `turmoil` seeds, exercising the harness itself: a message sent under a configured fault (drop, delay, reorder, partition) behaves exactly as that fault specifies, across every seed in the set - not just one.
- **Conceptual tier (Coachgremlin):** confirms the learner can explain why a single arbitrary seed wouldn't have been sufficient evidence the harness works, and that the harness's own API doesn't accidentally make a class of fault impossible to configure.

## Takeaway

A `turmoil`-based network-fault-injection harness template: reusable scaffolding for testing any future async Rust project's behavior under packet loss, latency, reordering, and partitions. Packaged by Coachgremlin once the rubric is met.

## Stop condition (placeholder)

The learner's harness passes the deterministic tier across the full published seed set, and Coachgremlin confirms the conceptual tier, per the gate above.

---

> **Skeleton only.** This module has a decided question, arc position, gate shape, and takeaway shape. It has no authored exercise, fixture, or rubric yet - that's Coachgremlin's job, run later. See [`modules/README.md`](../README.md) for workshop-wide status.
