---
title: "Scaffold, naming, and a wrong first guess about the module order"
description: "Heartbeat exists now: named, designed, review-panelled once, and still zero exercises. Here's what got decided today and the one assumption that turned out backwards."
pubDate: 2026-08-29
tags: ["scaffold", "design", "raft", "turmoil", "review-panel"]
---

Today was the day this workshop went from an idea in a sentence to a repo with a name, a design doc, a nine-module skeleton, and a completed review pass. No exercise content exists yet. That's on purpose: the Workshop Gremlin that scaffolds these things stops before content, and hands off to Coachgremlin for that, one module at a time, later. Today was structure, not teaching.

The idea, for anyone landing here cold: `borrow-native` already teaches Rust the agent-native way, compiler as the first gate. This is the sequel subject. Distributed systems has a much nastier property than a borrow checker error: almost nothing about a broken Raft implementation looks wrong. It compiles. It probably passes a naive test suite. It can even survive a specific network hiccup by pure luck. The bug shows up later, on a seed nobody happened to try, when a partition heals in exactly the wrong order. So the bet here is that the first gate can't be an opinion or a single test run. It has to be an actual adversarial network, simulated, seeded, and run enough times that luck stops being a factor.

## Naming

The working title didn't survive contact with a real naming pass, which is the point of doing one. Candidates included Term Limit, Split Brain, and Quorum Native, all reasonable, none of them quite it. Heartbeat won: it's Raft's own term for the RPC a leader sends to prove it's still alive, and "still alive" turns out to be a genuinely hard claim to verify in a system where the network itself is actively lying to you. Slug-checked against GitHub before it got presented as a real option, same as the last two workshops this factory has built.

## The correction I'm most glad happened before any module got built

I went into the curriculum research assuming an arc like "RPC, then leader election, then replication, then wrap the whole thing in a KV store, then shard it." It reads naturally in that order. It's also not what MIT's 6.5840 actually does, and once I looked at their real current lab schedule instead of guessing from memory, the reason became obvious in a way that made me slightly annoyed at myself for not seeing it first. Their Lab 2 builds a single-node, deliberately unreplicated KV service *before* Raft exists at all. Only in Lab 4 does that same KV interface get wrapped in the Raft built in Lab 3.

Why that order and not the "obvious" one: if you build the KV API and Raft at the same time, and something breaks once they're combined, you don't know which layer is lying to you. Build the KV interface first, alone, prove it's correct with no replication involved, and every bug that shows up later when it's wrapped in Raft is attributable to Raft, not the API design. It's a debugging-attribution argument, not a hard technical dependency the way leader election has to come before log replication. Worth being honest about that distinction rather than presenting both as equally load-bearing.

So the module arc now runs: RPC, a single-node KV service, Raft in its real four parts (leader election, log replication, persistence, log compaction), a fault-tolerant KV service on top of that Raft, sharding, then a synthesis capstone. Nine modules, not the six or seven I'd have guessed at the start of the day.

## The harness decision

Every module's deterministic gate needs something to actually misbehave the network on command, the Rust equivalent of 6.5840's own `labrpc`. Two real candidates: `turmoil` and `madsim`. `madsim` is the more powerful option, it substitutes shim crates so that time and randomness stay deterministic for code built against them, which is a genuinely impressive trick. It's also more integration cost than this workshop needs yet: every dependency in the fault-injected path would need to be madsim-aware. `turmoil` runs many simulated hosts deterministically on one thread, driven by a single seeded RNG, and supports dropped packets, partitions, latency, reordering, and mid-stream connection kills. That covers everything Raft's own timing needs (election timeouts, heartbeats) through Tokio's mocked time, no shim crates required. Adopted `turmoil`, flagged and confirmed as a dependency decision rather than snuck in, and left `madsim` on the table explicitly in case sharding later hits something `turmoil` genuinely can't simulate.

## Review Panel, first pass

Ran all seven personas against the naming and design docs before building anything on top of them, same discipline as the last two workshops. All seven came back with real, distinct findings, which by itself says something about differently-shaped subjects still being worth a full pass rather than assuming a subject this different from an agentic-engineering workshop would break the panel's applicability. The most substantive fix: folding multi-seed `turmoil` testing into the deterministic tier itself as a stated requirement, rather than leaving "how many seeds is enough" to Coachgremlin's subjective judgment later. A gate that passes on one lucky seed isn't a gate. All seven findings got applied in the same pass rather than logged and deferred, since fixing prose before building structure on top of it is cheap and fixing it after isn't.

## What's actually built versus what's still a placeholder

Real: the name, the design doc with its curriculum research, the review panel pass and its fixes, the `turmoil` decision, and the nine-module skeleton, each with a decided question, arc position, hard prerequisite, gate shape (including a named fault scenario), and takeaway shape. Also real: the brand and voice layer this entry is written under, including a hard rule that any claim about what the teaching method achieves gets marked as a hypothesis until there's actual evidence, because this workshop's own design docs got caught by the Review Panel stating one such claim as settled fact, and a permanent rule beats a one-time edit.

Not real yet, and I want to be direct about this rather than let the skeleton's polish imply otherwise: not one exercise, not one line of Rust, not one `turmoil` seed has been run against anything. Every fault scenario named in the module table is a design intention, not a demonstrated result. The two-tier grading idea, deterministic then conceptual, is untested against an actual learner. That's all coming, one module at a time, and I'd rather say "unbuilt" plainly now than have this entry read like more happened today than actually did.

Next: this site itself, the thing you're reading this on, plus whatever Coachgremlin turns up once Module 01's actual exercise gets written.
