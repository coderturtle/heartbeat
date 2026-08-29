# Modules

Heartbeat's spine is MIT 6.5840's own real lab order, translated to Rust: **RPC → a single-node service → Raft (leader election → log replication → persistence → log compaction) → a fault-tolerant service on top of Raft → sharding**, then a synthesis capstone. Work through them in order. Modules 03-08 each state a hard prerequisite on an earlier module; skipping ahead means hitting failures the workshop hasn't equipped you to diagnose yet.

This arc is anchored to [MIT 6.5840](https://pdos.csail.mit.edu/6.5840/) ("Distributed Systems," formerly 6.824) - the field's dominant free teaching resource, and the only reason this workshop doesn't have to invent a lab sequence from scratch. See [`docs/workshop-design.md`](../docs/workshop-design.md) for the full curriculum-anchor research, including a real correction this workshop's own research caught: 6.5840's current lab order builds the service interface *before* Raft, not after, which this arc follows rather than the more obvious-sounding "build Raft, then wrap it in a service."

Every module's core exercise is run through your own coding-agent harness (Claude Code, Codex, or equivalent). Two gates, not one: a **deterministic tier** (`cargo test` green across both `turmoil`-simulated network-fault seed sets, not just one - see [`docs/workshop-design.md`](../docs/workshop-design.md) for why a single set isn't enough) and a **conceptual tier** (Coachgremlin, checking protocol reasoning a passing test suite can't fully verify). See the top-level README and `docs/workshop-design.md` for the full thesis.

**Hands-on by design, not passive text.** No module here completes by reading it. Every module states a required gate: an artifact you produce or an action you're observed doing, checked first mechanically (against a real, adversarial simulated network), then conceptually. If a module ever reduces to "read this, then move on," that's a defect. Every gate also has a stated **takeaway**: you keep something reusable, not just proof you did the exercise.

> **Content status:** all nine modules are skeleton only - question, arc position, gate shape (including a named fault scenario per module), and takeaway shape decided; the actual exercise, fixture code, and rubric are Coachgremlin's job, run later, one module at a time, per the Workshop Gremlin's own Completion Condition (it stops before content exists). First Workshop Review Panel pass complete against the naming + design docs: `docs/review-panel/2026-08-29-initial-design.md`. Deterministic-gate design revised through three doubt-driven-development cycles: `docs/decisions.md`. Reseeded 2026-08-29 around one real product, `Checkout`, after a staff-engineer critique found the original generic-KV-store vehicle undifferentiated - see "One shared project," below.

## One shared project: `Checkout`, not eight toy fixtures

Every module builds toward one real product: **`Checkout`**, a distributed lock/session-ownership service - `Checkout(resource, holder, lease_duration)` grants exclusive, time-bounded ownership of a resource; `Renew` extends it; `Return` releases it early; an unrenewed lease expires on its own. This isn't an arbitrary choice: it's the exact shape of a real problem this factory's own `git-contract.md` already documents (two concurrent agent sessions can't safely hold the same branch/worktree at once - `Checkout` is that coordination primitive, built from scratch).

**This deliberately doesn't copy `borrow-native`'s `relay` shape**, and that difference is worth stating rather than silently diverging: `relay` adds one real feature to the same project every single module, because Rust's own concept arc is a chain where each concept is genuinely required to build the next. Raft's protocol modules don't have that property - leader election, log replication, persistence, and log compaction are generic consensus mechanics that don't know anything about locks or leases; they replicate opaque log entries for *any* application built on top, the same way 6.5840's own Lab 3 (generic Raft) precedes Lab 4 (the specific KV application). So `Checkout`'s build-out is honestly uneven:

| # | Module | What it adds to `Checkout` |
|---|---|---|
| 01 | RPC Over an Unreliable Network | The RPC layer, exercised with `Checkout`'s own message shapes (`CheckoutRequest`/`CheckoutResponse`) even though no real checkout logic exists yet |
| 02 | A Single-Node Checkout Service | The real `Checkout`/`Renew`/`Return`/`Status` API against a single in-memory node: exclusivity, lease expiry, no replication yet |
| 03-06 | Raft (leader election, log replication, persistence, log compaction) | The generic Raft engine `Checkout` will run on starting Module 07 - deliberately not `Checkout`-specific |
| 07 | Fault-Tolerant Checkout Service on Raft | Module 02's API wrapped in Module 06's Raft engine: requests replicated, surviving a leader failover mid-request |
| 08 | Sharded Checkout Service | Resources partitioned by namespace across many replica groups, scaling to a large fleet of concurrent agent sessions |
| 09 | Synthesis capstone | A real, seeded bug in the accumulated `Checkout` service |

## The arc

| # | Module | Hard prerequisite | The question it answers | Named fault scenario (deterministic tier) |
|---|---|---|---|---|
| 01 | [RPC Over an Unreliable Network](01-rpc-over-unreliable-network/README.md) | none (Rust fluency assumed) - **not a warm-up**, see that module's own callout | How do I build an RPC layer, and the harness that can lie to it convincingly? | The harness itself, under test: dropped/delayed/reordered `CheckoutRequest`/`CheckoutResponse` messages arrive exactly as configured, across both seed sets |
| 02 | [A Single-Node Checkout Service](02-single-node-checkout-service/README.md) | 01 | What does an exclusive-ownership API need to guarantee before I even think about replicating it? | Concurrent checkout requests under injected latency/reordering never grant the same resource to two holders at once, and every lease correctly expires |
| 03 | [Raft: Leader Election](03-raft-leader-election/README.md) | 01 | How does a cluster agree on exactly one leader, even when the network actively works against it? | A `turmoil` partition isolating the current leader; exactly one new leader is elected in the majority partition, never two in the same term |
| 04 | [Raft: Log Replication](04-raft-log-replication/README.md) | 03 | How does a leader get every follower's log to match its own, even a follower that's fallen behind? | A follower that missed several entries catches up correctly after a partition heals, log-matching property intact |
| 05 | [Raft: Persistence](05-raft-persistence/README.md) | 04 | What has to survive a crash, and what happens if it doesn't? | A crash-and-restart mid-replication recovers exactly the pre-crash persisted state, never re-votes in a decided term |
| 06 | [Raft: Log Compaction & Snapshots](06-raft-log-compaction-snapshots/README.md) | 05 | How do I discard old log entries without losing a lagging follower? | A follower lagging past the compaction point catches up via `InstallSnapshot`, not a replay of discarded entries |
| 07 | [Fault-Tolerant Checkout Service on Raft](07-fault-tolerant-checkout-service/README.md) | 02 and 06 | How do I wrap a working checkout/lease API in Raft without either layer leaking its bugs into the other? | The `Checkout` service stays linearizable across a simulated leader failover mid-request - no granted lease disappears, no denied checkout silently becomes granted |
| 08 | [Sharded Checkout Service](08-sharded-checkout-service/README.md) | 07 | How do I split one `Checkout` service into many, without a resource's lock ever belonging to zero or two of them? | A shard migration in progress, combined with a partition on the source or destination group, never leaves a resource ownerless or double-owned |
| 09 | [Synthesis capstone](09-synthesis-capstone/README.md) | all of the above | Given a real bug in the `Checkout` service you built, which concept is actually the root cause? | Every module's `turmoil` suite green again on the fixed program, plus a written, correct root-cause diagnosis |

## Gate tiers (every module uses this vocabulary)

| Tier | What it is |
|---|---|
| Deterministic (primary) | `cargo test` green across both a published **practice** seed set and a held-out **grading** seed set - passes or it doesn't, no judgment call, but never proof of correctness, only that the implementation resisted every targeted fault schedule tried against it. See `docs/workshop-design.md` for the full design (and the three doubt-driven-development cycles that hardened it). |
| Conceptual (secondary, Coachgremlin) | Protocol reasoning a passing multi-seed suite still can't fully verify: does the learner's explanation make a falsifiable prediction Coachgremlin can check, not just a plausible-sounding one? |

A green deterministic tier is necessary, never sufficient on its own - the same discipline `borrow-native`'s Coachgremlin learned the hard way, raised a level here: a Raft implementation can pass a seed set by chance in a way a compiler error never can, which is exactly why the deterministic tier requires two independently-generated, disjoint seed sets, not one published set. See [`docs/workshop-design.md`](../docs/workshop-design.md) and this project's Coachgremlin reference (`gremlins/coaching/coachgremlin.md` in the Hekton operating-model repo)'s Workflow step 3.

## What you keep

Each module's gate produces a takeaway, not just proof: a real, keepable artifact.

| # | Module | Takeaway |
|---|---|---|
| 01 | RPC Over an Unreliable Network | A `turmoil`-based network-fault-injection harness template, reusable on future async Rust projects |
| 02 | A Single-Node Checkout Service | An exclusive-ownership API checklist: idempotent checkout, renewal races, lease-duration tradeoffs |
| 03 | Raft: Leader Election | A leader-election diagnostic playbook (term/vote/timeout reasoning) |
| 04 | Raft: Log Replication | A log-matching-property diagnostic checklist |
| 05 | Raft: Persistence | A "what actually needs to survive a crash" checklist, generalizable beyond Raft |
| 06 | Raft: Log Compaction & Snapshots | A snapshot-boundary decision guide |
| 07 | Fault-Tolerant Checkout Service on Raft | A layering playbook: keeping a service's API and its consensus layer independently testable |
| 08 | Sharded Checkout Service | A shard-rebalancing/ownership-handoff checklist |
| 09 | Synthesis capstone | A personal distributed-systems diagnostic playbook compressing the whole arc |

## Why this order

This is this workshop's own editorial synthesis, anchored to MIT 6.5840's real lab order rather than invented from scratch (see `docs/workshop-design.md`'s curriculum-anchor research for the full reasoning and the correction that shaped it). Leader election comes before replication because there's nothing to replicate without a leader. Replication comes before persistence because persistence exists to survive a crash mid-replication. Persistence comes before compaction because compaction discards persisted state, so the thing being discarded has to be reliably saved first. The single-node `Checkout` service (Module 02) comes before Raft entirely, matching 6.5840's real order: building the interface without replication first means every bug surfaced in Module 07 is attributable to the Raft layer, not the API design - worth being honest that this is a debugging-attribution argument, not a hard technical dependency the way 03→04→05→06 are. Sharding (08) is last among the core modules because it assumes a working replicated `Checkout` service to shard in the first place.

## A note on Module 01

Module 01 is not a gentle warm-up. It has no *conceptual* prerequisite, but it's where you build the `turmoil`-backed network-fault-injection harness every later module's gate depends on - real engineering work with zero distributed-systems intuition yet to lean on. Budget real time for it.
