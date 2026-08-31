# Retro: Heartbeat's third real content-building dry run

Module 03 is the first module in this workshop's Raft core (03-06) and the
first whose fixture design itself went through doubt-driven-development
before any test or reference implementation existed - a genuinely different
starting point from Module 01 (dry run first, DDD found afterward it wasn't
enough) and Module 02 (same pattern, twice). This retro checks whether
front-loading DDD on the fixture actually paid off, plus two new findings
this dry run's own process surfaced.

## Did front-loading DDD on the fixture avoid Module 01/02's own pattern of a dry run finding what design review missed?

**Partially, and the way it didn't matters.** Three DDD cycles (single-model,
Codex, single-model) on the fixture *types and API shape* caught real,
serious bugs before any code existed on disk: a stale/buggy `RaftLog` that
would have reintroduced this project's own already-fixed `term_at`/
`prev_term_for` livelock, a non-re-randomized election timeout that broke
Raft's own liveness argument, an unconstructible observability channel, and
a dependency that turned out not to be available to this crate at all
(`rand`/`blake3` - `docs/completion-roadmap.md` had already found this once,
for a different reason, and cycle 3 caught the fixture design quietly
reintroducing it). All of that was real, and none of it would have been
caught by a dry run alone unless someone happened to write a naive attempt
that specifically tripped each one.

What DDD on the fixture's *types and API shape* could not catch, by
construction: whether a claim about a specific tool's runtime behavior
(this crate's pinned `turmoil` version's actual `partition`/`connect()`
semantics) was true. That's not a design-review question - the DDD skill's
own instruction for this artifact was explicitly "do NOT build, run, or
compile any code," which is correct for reviewing a design draft that
doesn't exist as real files yet, but means a *runtime behavior claim*
embedded in that design's own doc comments (the "held lock deadlocks under
partition" line) went unchecked by all three cycles and had to wait for an
actual dry run - with a real implementation, a real test, and in this case
a small standalone probe - to be found wrong. Worth naming precisely: DDD
on a design catches design-shaped bugs (wrong types, missing methods,
unstated contracts); it doesn't substitute for actually running code
against the real environment it will execute in, no matter how thorough the
review.

## A new finding this dry run's own process surfaced: a claim can be *plausible* and *wrong*, and only an empirical check tells you which

The "held lock across an outbound `connect()` deadlocks under partition"
claim was not a guess pulled from nowhere - it's a real, well-known Rust
anti-pattern (holding a mutex guard across an `.await` starves every other
task needing that lock), and in *production*, a TCP connect to an
unreachable host genuinely can hang for a long time on OS-level SYN
retries. The claim was wrong specifically for *this crate's pinned turmoil
version's own simulated model* of a partition, which fails a fresh connect
immediately rather than reproducing that real-world hang. Nothing about
this was unreasonable to assume; it was reasonable and untested, and a
5-line standalone probe (a separate scratch crate, one `#[test]`,
`turmoil::partition` then a timed `connect()`) resolved it in under a
minute once someone thought to check. The lesson for every future module
in this arc, not just this one: when a design's own claim rests on how a
specific *tool* behaves (not how the algorithm itself behaves), verify that
claim against the tool directly, the same way this project already
verifies claims about the algorithm against a real reference
implementation - don't let "it's a well-known general principle" substitute
for "I checked this specific case."

## A second finding, downstream of the first: a wrong empirical premise silently made a test vacuous

The partition test's original design isolated a fixed node id
(`node-0`), reasoning implicitly that *some* node needs to be isolated to
force the majority to hold a fresh election, and node 0 was as good a
choice as any. But leadership in this design is genuinely load-bearing:
whichever node's own seeded RNG happens to draw the shortest initial
timeout becomes the first leader, with no reason to correlate with node id.
Isolating a fixed, arbitrary node id therefore only forces a fresh election
when that specific node *happens* to already be leading - on the 5 seeds
this suite runs, apparently never, since neither draft of attempt-naive's
own held-lock bug nor its eventual replacement (the vote-uniqueness bug)
were caught until the isolation target was made to track the *actual*
current leader, discovered live via the same wire-observation hook already
in place for the independent anti-gaming cross-check. This is the same
category of bug Module 02's own dry run found once already (a test whose
premise silently didn't hold for the scenario it claimed to cover) -
recurring in a new, Raft-specific shape rather than a surprise this
workshop hasn't seen before.

## Did the anti-gaming, wire-level cross-check earn its place?

**Yes, though not yet proven against a bug that specifically targets it.**
`assert_wire_claims_agree_with_transition_log` independently derives
leader-per-term uniqueness from the actual `AppendEntries` traffic observed
on the wire by the test harness's own accept loop, not from anything the
learner's code calls voluntarily - this closes the gap a purely
self-reported `RoleState`/`TransitionLog` mechanism would have (a learner
implementation that simply never called the observability hook would pass
the primary check by omission). Neither attempt in this dry run happened to
trip a *disagreement* between the two mechanisms specifically (attempt-good
agrees by construction; attempt-naive's vote-uniqueness bug is caught by
the primary transition-log check before the wire-level check would need to
disagree with it). This mirrors Module 02's own dry-run precedent exactly:
a mechanism can be real, correctly implemented, and not yet have been
proven against a bug that specifically requires it to be the one that
catches something - worth a fresh adversarial reviewer's attention, not
assumed sufficient just because it exists.

## What Module 04's own dry run should check fresh, not assume

Module 04 (log replication) reuses this module's `RaftLog`, `RaftNode`
struct, and transport - and will be the first module to actually populate
non-empty logs and exercise `entries_from`/`compacted_boundary`'s
documented-but-unenforced contract. Don't assume Module 03's "one of five
tests actually catches the injected bug" ratio generalizes - check fresh
whether Module 04's own new tests (log-consistency checks, commit-index
advancement) discriminate on their own first write, the same way this
retro found this module's own didn't. Also carry forward explicitly: before
writing a test whose design depends on how a specific tool (not the
algorithm under test) behaves under a specific fault, write a small
standalone probe of that tool's actual behavior first - this dry run's own
value came partly from doing that after the fact; doing it before would
have caught the same finding earlier and cheaper.

## Go/no-go on Module 03 (dry run alone)

**Go, pending the test suite's own DDD pass, not before** - matching this
retro's own prediction one section up that this workshop's dry-run-then-DDD
pattern would likely recur here in some shape.

## Addendum: a doubt-driven-development pass on the test suite found the same bug class recurring, one layer in

A single-model adversarial review of the test suite - given the file and its
governing contract, told explicitly not to extend courtesy to claims this
retro had already "confirmed" - found six real, confirmed issues. See
`grading.md`'s own addendum for the full list; the pattern worth naming here
is what kind of bug each one was.

**The most pointed finding recurred inside the exact test this retro's own
first finding was about, one assertion downstream.** This retro's "second
finding" above documented fixing the partition test's *scenario* (isolate
the live leader, not a fixed id). The DDD pass found the test's *liveness
assertion*, sitting right next to that fix, was independently vacuous for an
unrelated reason: it checked for a leader anywhere in the whole merged
history, which the pre-partition leader alone already satisfies before the
partition is even applied. Two vacuousness bugs in the same test, found in
two different passes, each real and each independently sufficient to make
the test unable to fail. Worth stating plainly: fixing a test's premise does
not imply its assertions were also checked with the same scrutiny - they are
separate claims, and this retro's own first fix left the second one standing
until a fresh reviewer looked at the artifact as a whole rather than at the
specific line that had already been flagged once.

**A second, structurally different finding: a contract requirement can have
zero test coverage even in a suite that looks complete.** "Election-timeout
jitter re-randomizes per attempt" is stated as a hard requirement in
`timer.rs`'s own doc comment and was never once tested - not weakly tested,
not indirectly implied, actually zero tests would have caught a learner
copying `self.rng` fresh out of a `Copy`-derived field every election
attempt, discarding the advanced state each time. This is a different
failure mode from the vacuous-assertion pattern above: those tests *ran*
and *could* fail, just not for the reason claimed; this requirement had no
test running against it at all. Both are real gaps a fresh reviewer finds by
asking different questions - "does this assertion actually prove what its
name claims" versus "is every stated requirement backed by *some* test" -
and this dry run's own process needed both questions asked before either
surfaced.

**A structural observation for future modules in this arc:** the
`DeterministicRng: Copy` derive that makes the re-randomization bug easy to
write by accident was a deliberate design choice from this module's own
fixture-API-split DDD cycles (Copy was needed so the RNG could be read out
of `&self`/`&Arc<Self>` without forcing interior mutability on every
learner). A provided type's own ergonomic choice can create exactly the
footgun its neighboring doc comment warns against - worth checking, for
Module 04-06's own provided types, whether an ergonomic affordance and a
correctness requirement are quietly pulling in opposite directions the way
they were here.

## Go/no-go on Module 03 (final)

**Go.** The fixture (post three DDD cycles) and test suite (post this
fourth DDD cycle, its own doubt-driven-development pass) are real and
internally consistent: a correct attempt passes all 6 tests stably across
repeated runs, a real, honest wrong attempt fails 3 of 6 for an understood,
verified-not-flaky reason - a materially stronger discrimination profile
than the 1-of-5 this retro's own dry run originally recorded, because the
safety check itself is now an absolute one rather than an interval-overlap
approximation. Every finding this module's combined dry-run-and-DDD process
surfaced (three fixture-design cycles, one dry-run pass, one test-suite DDD
cycle) is fixed and documented, not glossed over - including two cases,
named explicitly above, where a fix from one pass left a real gap for the
next pass to find, exactly the pattern this project's own precedent (Module
02's cycle 2 finding gaps in cycle 1's fixes) predicts will keep happening
and is not, on its own, a reason to distrust the process.
