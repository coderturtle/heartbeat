# Heartbeat

A correct-looking Raft implementation can pass every unit test and still lose committed data the first time a network partition heals badly. Heartbeat is a self-paced workshop that makes you build the network fault into the test, not just the code into the test - by building a real distributed lock service, not a toy.

## What this is

You already learned Rust the agent-native way, maybe via [`borrow-native`](https://github.com/coderturtle/borrow-native). This workshop teaches distributed systems the same way: build **`Checkout`**, a distributed lock/session-ownership service, from scratch, module by module, with your own coding-agent harness doing the typing and two gates checking the result. `Checkout` isn't an arbitrary toy - it's the exact coordination primitive this factory's own operating rules already need: two agent sessions can't safely hold the same branch or worktree at once, and `Checkout` is what arbitrates that, durably, even when the node holding the answer crashes mid-request.

Two gates check the result. First, a **deterministic gate**: your implementation runs against [`turmoil`](https://docs.rs/turmoil), a simulator that drops, delays, reorders, and partitions network traffic on command, across two independently-generated seed sets (one you can see, one you can't), not one. Second, a **conceptual check** from Coachgremlin, this workshop's teaching agent (a role you run yourself, inside your own harness, not a hosted service): can you explain *why* your implementation is safe, not just that it happened to survive every seed you were tested against.

The workshop's own name is Raft's own term for the RPC a leader sends to prove it's still alive. This workshop's bet is that "still alive" is a much harder claim to verify than it sounds, and the whole arc is built around actually checking it instead of assuming it.

**Who it's for:** agent-literate practitioners, comfortable with git, the CLI, and a coding agent, already fluent in Rust (ownership, borrowing, traits, async - the level `borrow-native` teaches to). Not a Rust-fundamentals workshop and not a true-beginner distributed-systems course. The one thing assumed unfamiliar is distributed systems itself.

## Prerequisites

- Comfortable with git, the CLI, and reading a diff.
- Fluent in Rust: ownership, borrowing, traits, `async`/await.
- Already using at least one coding-agent harness regularly, with one installed on your machine.
- Rust and `cargo` installed (`rustup`).

## How to start

```bash
git clone git@github.com:coderturtle/heartbeat.git
cd heartbeat
cat modules/README.md
```

Then work through `modules/` in order. Modules 03-08 each state a hard prerequisite on an earlier module; skipping ahead means hitting failures the workshop hasn't equipped you to diagnose yet.

> **Current status: Module 01 is real, Modules 02-09 are still skeleton only.** [Module 01, RPC Over an Unreliable Network](modules/01-rpc-over-unreliable-network/README.md) has a working exercise you can actually run today, dry-run against a correct and a naive attempt. The rest have a decided question, arc position, gate shape (including a named fault scenario), and takeaway shape - see [`modules/README.md`](modules/README.md) - but no authored exercise yet. Watch `docs/build-log/` for progress, or [open an issue](https://github.com/coderturtle/heartbeat/issues) to ask.

## How the modules connect

RPC has no prerequisite, but it's not a warm-up: it's where you build the network-fault-injection harness every later module depends on, exercised with `Checkout`'s own message shapes from day one. A single-node `Checkout` service comes next, deliberately unreplicated, so a bug found later when it's wrapped in Raft is attributable to the Raft layer, not the API. Then Raft itself, in MIT 6.5840's own real order: leader election, log replication, persistence, log compaction - deliberately generic, since Raft doesn't know or care what it's replicating. A fault-tolerant `Checkout` service wraps that complete Raft around the earlier interface. Sharding splits one replicated service into many, scaling to a large fleet of concurrent agent sessions. A synthesis capstone closes the arc: given a real bug in the `Checkout` service you built, diagnose which concept is actually the root cause. Full arc, gate tiers, and the curriculum research behind it: [`modules/README.md`](modules/README.md).

## What you keep

Every module leaves you with something, not just a passed check: a reusable network-fault-injection harness, an API-design checklist, a leader-election diagnostic playbook, a log-matching checklist, a "what needs to survive a crash" checklist, a snapshot-boundary decision guide, a layering playbook, a shard-ownership checklist, and a personal distributed-systems diagnostic playbook tying it all together. See [`modules/README.md`](modules/README.md#what-you-keep) for the full list.

## The teaching method

Distributed systems has a property Rust's own compiler doesn't: a broken implementation can pass a test by luck, not just by correctness, because the specific fault that would expose the bug simply didn't happen to fire on that run. That's why this workshop's deterministic gate requires green across two independently-generated `turmoil` seed sets, not one you can see and tune against, and why Coachgremlin's conceptual tier exists on top of it: to catch design choices that happen to survive every seed tried so far without actually being correct. This is a stated design bet, not a proven finding yet - no module content exists to test it against a real learner, and three rounds of adversarial review found real problems with earlier versions of this same gate design before it landed here (see `docs/workshop-design.md`). See [`docs/workshop-design.md`](docs/workshop-design.md) for the full reasoning, the MIT 6.5840 curriculum research behind this arc, and what's still open.

## Build in public

This workshop's own build will be published as a dated journal at `coderturtle.github.io/heartbeat` once the site is built and the first deploy is human-confirmed: the maintainer's record of building the workshop and its reusable Gremlin tooling at the same time, written deliberately rather than auto-generated from session logs.

## Something wrong?

This is early and imperfect by design. If a module reduces to "read this, then move on" instead of a real gate, or a link here is broken, [open an issue](https://github.com/coderturtle/heartbeat/issues).

## Key docs

- [Workshop Design](docs/workshop-design.md): audience, format, MIT 6.5840 curriculum research, deterministic-gate teaching method, full module arc
- [Maintainers](docs/maintainers.md): internal/agent-facing docs, classification, documentation contract
