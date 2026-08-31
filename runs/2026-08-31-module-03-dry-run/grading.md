# Grading record: Module 03 dry run (2026-08-31)

Coachgremlin's third real content-building dry run, and the first for the
Raft core (Modules 03-06). Both attempts run against
`fixtures/checkout/tests/module_03_raft_election.rs` (5 tests), against the
provided `fixtures/checkout/src/raft/{types,timer,connector,transport}.rs`
that went through three doubt-driven-development cycles before either
attempt was written (see `docs/decisions.md`'s 2026-08-31 entry for that
process's own findings).

## attempt-good

A correct reference implementation of `RaftNode<C: Connector>` (Mutex-based
interior mutability, a single background election-timeout task, `tokio::task::JoinSet`
for concurrent per-peer RPC fan-out during elections and heartbeats). Passes
all 5 tests, stable across 3 repeated full-suite runs. `cargo clippy --tests
-- -D warnings` clean for every file this dry run touched (`raft/node.rs`,
`raft/transport.rs`, `tests/module_03_raft_election.rs`) - the crate-wide
clippy run still shows the pre-existing 9 errors from Module 01/02's own
unimplemented `todo!()` stubs, unrelated to this module and already
documented (`docs/decisions.md`'s 2026-08-31 entry on Module 02, and the
`rust-ci.yml`-red-on-main question left open for a human).

## attempt-naive-unconditional-vote-grant

One deliberate, honest bug: `handle_request_vote` forgets Figure 2's "at
most one vote per term" restriction entirely - grants to any candidate with
an acceptable term and an up-to-date-enough log, even one this node already
voted for someone else in the same term. Fails exactly
`a_partitioned_minority_leader_never_shares_a_term_with_the_majoritys_new_leader`
(4/5), stable across 3 repeated runs. `cargo clippy --tests -- -D warnings`
clean for the same three files - this is a domain-logic/state-correctness
bug with no clippy lint that could plausibly express it, matching Module
02's own retro finding that clippy and the test suite catch different bug
classes and neither substitutes for the other.

Notably, this bug does **not** fail the two no-fault tests
(`a_three_node_cluster_elects_exactly_one_leader_per_term`,
`election_succeeds_under_injected_latency`) - a genuinely contested,
concurrent split vote (the specific scenario where two candidates could
both cross majority threshold via double-granted votes) needs the
kind of multi-node interaction the partition scenario's isolate-then-
re-elect sequence reliably produces, but isn't guaranteed by an
uncontested no-fault election on these 5 seeds. This is real, useful
signal about which of this suite's tests actually exercises the vote-
uniqueness safety property - see `retro.md` for what this implies.

## A real empirical correction found during this dry run, before either
## attempt's own bugs were evaluated

A standalone probe (`turmoil::partition` between a fresh client and a
listening host, then a `connect()` attempt under a 5-second `tokio::time::timeout`)
found that this crate's pinned `turmoil = "=0.7.2"` fails a new `connect()`
to an already-partitioned host **immediately** (`ConnectionRefused`, 0ns
elapsed) - it does not hang. `node.rs`'s and `transport.rs`'s own doc
comments, written before this was checked, claimed the opposite: that
holding a state lock across an outbound `connect()`/RPC call would
"deadlock the instant a peer is partitioned away." That claim was never
verified against the actual pinned `turmoil` version's real behavior before
being written down as a testable requirement - and once it was written
directly into a test's own design (an earlier draft of the partition test
isolated a *fixed* node id, assuming holding-the-lock-across-a-hanging-
connect would visibly wedge a majority-side candidate), it produced a
vacuous test: neither attempt's own held-lock behavior (attempt-good has
none; an earlier attempt-naive draft held a `tokio::sync::Mutex` guard
across the entire election fan-out) affected the test's pass/fail outcome
at all, since `connect()` to a partitioned peer simply fails fast regardless
of whether the caller is holding a lock across it.

Both `node.rs`'s and `transport.rs`'s doc comments were corrected to state
what's actually true (the anti-pattern is real in production, where a real
TCP connect to an unreachable host can hang for a long time on OS-level
retries, but this simulated harness's own `partition` semantics can't force
that specific failure into a visible test outcome) rather than a claim that
sounded plausible but was never checked. The partition test itself was
separately found to have a second, independent vacuousness bug: it isolated
a fixed node id (`node-0`) rather than whichever node was actually leading -
since leadership is not more likely to land on any particular node id, this
made the test's own core scenario (force the majority to hold a *fresh*
election) contingent on node 0 happening to already be leader, which none
of the 5 seeds' outcomes required. Fixed by tracking the live leader via the
same wire-observation hook already used for the anti-gaming cross-check, and
isolating that node specifically - confirmed to actually change the test's
observed behavior (it now reliably catches the vote-uniqueness bug once one
was substituted in) rather than assumed fixed from reading the diff alone.

## What this dry run is and isn't evidence of

**Is:** confirmation that Module 03's fixture (post three DDD cycles) and
test suite are real and internally consistent - a correct attempt passes
everything, a real, honest wrong attempt fails at least one gate criterion,
and the reason it fails is understood, documented, and stable across
repeated runs.

**Is also:** a second, independent confirmation of this workshop's own
established discipline that a claim about *why* a test should work must be
checked against the actual tool/library behavior, not just written down
because it sounds right - matching Module 02's own dry run finding that a
`matches!`-only assertion looked like it tested an effect but didn't, and
this project's own private-implementation history of a `term_at` vs
`prev_term_for` bug slipping through until directly tested.

**Isn't:** a claim that every test in this suite discriminates against
every plausible bug - only one of five tests caught the one bug actually
injected here, and `retro.md` names this as a real, current gap rather than
glossing over it.
