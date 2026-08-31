# Module 03 Rubric: Raft Leader Election

Shared with the learner before the attempt, per Coachgremlin's own workflow. Criteria are property-phrased (an observable fact about the finished code), not technique-named (the fix itself stated outright).

| # | Criterion | Tier |
|---|---|---|
| 1 | `cargo test --test module_03_raft_election` is green, across every seed in the published sets. | Gate, deterministic |
| 2 | `cargo clippy --tests -- -D warnings` is clean. | Gate, deterministic |
| 3 | The diff touches only `src/raft/node.rs` - not `types.rs`, `timer.rs`, `connector.rs`, `transport.rs`, or `tests/module_03_raft_election.rs`. | Gate, anti-gaming |
| 4 | At most one node ever records a Leader transition for a given term, for the life of the run - checked as an absolute property (any two nodes, at any two points in the run), not merely "no two leaders' tenures overlap." | Scored, conceptual |
| 5 | Election-timeout jitter is redrawn from the node's own owned RNG on every new election attempt - not a value computed once at construction and reused, and not a value independently derivable without consuming that RNG's advancing state. | Scored, conceptual |
| 6 | Any RPC or reply carrying a term higher than the node's own current term causes it to become a follower and adopt that term - checked regardless of the node's role at the time (a candidate or a leader must step down exactly the same way a follower would). | Scored, conceptual |
| 7 | `new` spawns nothing; every task that runs continuously (the election timer, and anything else) is spawned from `start`, using a cloned `Arc<Self>` or equivalent - not from `new`, and not from a place a re-invoked `sim.host` factory closure would skip on a later `bounce()`. | Scored, conceptual |
| 8 | The election-restriction rule (§5.4.1: a candidate's log must be at least as up-to-date as a voter's before that voter grants it a vote) is implemented in `handle_request_vote`, even though this module's own deterministic tier cannot independently verify it (every log is empty throughout Module 03 - see criterion 8's own note below). | Scored, conceptual |

## Why criterion 4 is scored as an absolute property, not an interval-overlap one

A real, evidenced finding from this module's own doubt-driven-development pass on its test suite (`runs/2026-08-31-module-03-dry-run/grading.md`'s addendum): an earlier version of this exact check tracked *open* leadership intervals, clearing a node from a term's "current leaders" set the instant it transitioned away. Two different nodes recording `Leader` for the same term in sequence - the first stepping down before the second is ever recorded - is a real, absolute Raft safety violation regardless of whether their tenures overlap, and the interval-based check couldn't see it. This is exactly the shape of violation a missing one-vote-per-term restriction produces (a dry-run naive attempt with exactly that bug passed the interval-based check, 4 of 5 times, before this fix). Coachgremlin checks the absolute property directly now, and a learner should be able to state why "sequentially disjoint" isn't the same guarantee as "at most one, ever."

## Why criterion 5 is scored, not left to "the tests will catch it"

Also from this module's own dry run: the provided `timer::DeterministicRng` derives `Copy` (deliberately, so it can be read out of `&self`/`&Arc<Self>` without forcing interior mutability on every learner) - which makes "copy `self.rng` fresh inside your election-starting logic every time, discarding whatever state it had advanced to" the path of least resistance to write by accident, and none of this module's own no-fault or single-run tests can distinguish that from correct behavior (a constant, per-node timeout still elects *a* leader, just never re-randomizes on a genuine split vote). `a_split_vote_still_elects_a_leader_eventually` (30 seeds, an even-sized cluster) is the closest a black-box test can get to enforcing this indirectly; criterion 5 exists so Coachgremlin checks the actual code, not just whether that one probabilistic test happened to pass.

## What "explain why" means for criterion 8

Not "the test suite passes" - it can't, for this specific rule, in this module. The learner should be able to state, in their own words, why a candidate whose log is behind a majority's must not be allowed to win even if it collects enough votes by other means (a delayed-write scenario where the up-to-date data would be lost if that candidate became leader), and why this module's own test suite - where every node's log is empty throughout - cannot exercise that failure mode at all. Module 04 (log replication) is expected to be the first point where a real, divergent log makes this rule's absence independently, mechanically detectable; naming that gap honestly now, rather than pretending criterion 1's green tests already cover it, is itself part of what this criterion checks for.
