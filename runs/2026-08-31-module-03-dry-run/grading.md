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

## Addendum: a doubt-driven-development pass on the test suite itself

Everything above was the dry run's own record, written and trusted before a fresh
adversarial reviewer ever looked at the test suite itself - a real process gap, given
this project's own established practice of not treating a dry run's two-attempt
discrimination as sufficient on its own. A single-model review, given the full test
file and its governing contract as its artifact, found six real, confirmed issues -
including one that recurred in the *same* test the original dry run's own process had
already found a bug in, one assertion further downstream:

**`assert_at_most_one_leader_per_term` checked a strictly weaker property than its own
name.** It tracked "open" leadership intervals, clearing a node from a term's set the
moment it transitioned away - so two nodes recording `Leader@5` sequentially (the
first stepping down before the second is recorded) were invisible to it, even though
that is a real, absolute Raft safety violation regardless of overlap. This is exactly
the failure mode a missing one-vote-per-term restriction produces. Fixed to an
absolute per-term check: the set of nodes that *ever* recorded `Leader` for a term
must never exceed one, full stop.

**The partition test's own liveness assertion could never fail.** The original dry
run already found and fixed this test's core scenario being vacuous once (isolating a
fixed node id instead of the actual leader). This cycle found the *fix's own
assertion* was still vacuous, for a different reason: `assert_a_leader_was_elected`
checks the *entire* merged history for any `Leader` event at all - which the
pre-partition leader itself already satisfies, before the partition is even applied.
Nothing in the test required a *new* leader, in a *higher* term, on a *different*
node, to emerge afterward - the property the test's own doc comment claims to check.
Fixed by tracking the isolated node's own last known term and requiring a leader
transition elsewhere with a strictly higher term.

**Contract requirement "election-timeout jitter re-randomizes per attempt" had zero
test coverage.** Worse: `RaftNode`'s `rng: DeterministicRng` field derives `Copy`
specifically so it *can* be copied out of `&self`/`&Arc<Self>` without needing
interior mutability - but this makes the exact bug ("copy `self.rng` fresh inside
`start_election` every time, discarding the advanced state, so every attempt draws
the identical timeout") the path of least resistance for a learner to write by
accident, and none of the original five tests would have caught it: the lowest-seeded
node would just win every election, every time, indistinguishably from correct
behavior. Added `a_split_vote_still_elects_a_leader_eventually` (30 seeds, a 4-node
cluster so an even split is topologically possible) as the closest a black-box event
log can get to enforcing this - it cannot prove re-randomization occurred, only that a
genuine repeated-split-vote deadlock would eventually surface as a bounded-window
liveness failure across enough seeds if it didn't.

**The determinism test only ever exercised one hardcoded seed**, skipped the suite's
other safety assertions on both runs, and never checked that *different* seeds
produce *different* outcomes - only that identical seeds produce identical ones. Fixed
to loop over the full seed set (running every assertion each time) and added an
explicit cross-seed distinctness check.

**Background-task panics were silently swallowed.** `RaftNode::start()`'s own doc
comment promises the returned `JoinHandle`s let the harness "detect a panicked
background task instead of silently reading 'no leader elected'" - the harness
captured them as `let _handles = node.start();` and never touched them again. Without
`tokio_unstable` (confirmed absent from this repo), a panicked spawned task doesn't
crash the runtime; a node whose only background task panicked would fail silently,
misdiagnosed as "too slow" rather than "crashed." Fixed to spawn a watcher per handle
that records any unexpected completion or panic, with a new assertion checking that
collector is empty.

**Two doc-comment claims about `turmoil`'s partition behavior were still overbroad**
even after the original dry run's own correction. That correction established a fresh
`connect()` to an already-partitioned host fails fast (true, confirmed empirically).
But it then claimed the whole held-lock-across-an-outbound-call anti-pattern "isn't
forceable from this test" - overstated: an *already-established* connection, with a
reply silently dropped mid-flight by the same partition, still hangs for real, since
neither `call_append_entries` nor `serve_one_rpc` has a timeout. Corrected both
`node.rs` and `timer.rs`'s doc comments to state the narrower, accurate claim.

Smaller harness bugs fixed the same pass: the accept loop treated any `listener.accept()`
error as "this node is done" (`return Ok(())`) instead of propagating it; a failed
`sim.run()` discarded the transition history that would have been the actual
diagnostic, and its panic message named "a panic above" when the dominant failure mode
(an unmet liveness condition) never panics anywhere; `election_succeeds_under_injected_latency`'s
chosen latency band barely differed from `turmoil`'s own ambient default already present
in every other test, making it close to a duplicate of the plain no-fault test; a
`Sim`/receiver drop-order hazard (currently inert, but one `tokio` internals change
away from a real one).

All fixes re-verified end to end against the existing dry run: `attempt-good` passes
all 6 tests (one more than before - the new split-vote stress test), stable across
repeated runs; `attempt-naive` (the same single deliberate bug: forgot Figure 2's
one-vote-per-term restriction) now fails 3 of 6, up from 1 of 5 - the strengthened
absolute safety check now catches the bug directly, in the no-fault and
injected-latency tests too, rather than needing the partition scenario's specific
interaction to surface it as a liveness symptom.

**What this addendum is evidence of:** a dry run's own two-attempt discrimination
check is real signal but not sufficient on its own for the test suite any more than
for the fixture it tests - this project's own established finding for Module 02,
recurring here in a new shape. Notably, one of this cycle's findings was a bug in the
*previous* cycle's own fix (the partition test's liveness check, one assertion past
where the earlier fix landed) - a fresh reviewer catches this precisely because it
never saw the reasoning behind the first fix, only the artifact the fix produced.
