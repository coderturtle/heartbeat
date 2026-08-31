# Coachgremlin's dry run: Module 02, A Single-Node Checkout Service

Second real content-building dry run for Heartbeat, per the same evidenced practice Module 01 used: a correct attempt and a deliberately naive, honest (non-adversarial) attempt, both against the same stubbed exercise (`fixtures/checkout/src/checkout.rs`'s `CheckoutService::canonicalize`/`handle`/`status`), both run against the same provided test suite (`fixtures/checkout/tests/module_02_checkout_service.rs`). Module 01's own retro explicitly warned not to assume its "clippy catches what tests don't" finding would generalize - this dry run checked that fresh, and it doesn't (see below).

## Step 3: Observe the attempt

**`attempt-good/`** (`diff.patch`): a straightforward, correct implementation - real `ResourceState`/`ActiveLease`/`DedupEntry` types keyed by `(client_id, resource)` for dedup and by canonical `resource` for lease state, generation fencing checked before any mutation, expiry derived lazily from the injected clock on every call (`handle` and `status` alike), lease duration capped identically on `Checkout` and `Renew`. `cargo test --test module_02_checkout_service`: 20/20 pass. `cargo clippy --tests -- -D warnings`: clean.

**`attempt-naive-client-scoped-dedup/`** (`diff.patch`): identical in every respect except the dedup map's key - `HashMap<String, DedupEntry>` keyed by `client_id` alone, instead of `HashMap<(String, String), DedupEntry>` keyed by `(client_id, resource)`. A real, honest mistake: the module's own learning objectives describe the key as "`(client_id, resource, sequence number)`" in prose, and a first-time implementer who reads "keyed by client and sequence" without weighting `resource` equally could plausibly drop it, especially since the bug is invisible for the overwhelmingly common case of a client that only ever holds one resource at a time. `cargo test`: 19/20 pass - see below for the one exception and a genuine correction that had to happen first. `cargo clippy --tests -- -D warnings`: **clean**.

## Step 3 (continued): a real, self-caught test-design bug, found before this dry run's actual finding could even surface

Running `attempt-good` against the test suite the first time surfaced **two test failures against the correct implementation** - not a bug in the reference logic, a bug in the tests themselves, caught only because a genuinely correct attempt was run through them before trusting any result:

1. `a_lease_survives_until_its_duration_elapses_then_the_resource_is_reissuable` reused the same `(client_id="bob", resource, sequence=1)` triple for two calls meant to represent two different points in time. Dedup correctly replayed the first call's cached denial for the second, since nothing about the request's identity had changed - a real property working exactly as specified, exposing that the test itself, not the implementation, was wrong. Fixed by giving the second call `sequence=2`.
2. `a_racing_renew_and_return_never_leave_the_resource_in_a_corrupted_state` fired a `Renew` (sequence 2) and a `Return` (sequence 3) concurrently under the *same* `client_id`. Depending on thread-scheduling luck, whichever call the shared `Mutex` serialized second could see the other's higher sequence already cached and correctly reject as `StaleSequence` - again the dedup mechanism working as specified, not a real concurrency bug, exposing that sequence numbers express one client's own strict retry order and can't be reused to represent two calls meant to race. Fixed by using two different `client_id`s (`alice-renewer`, `alice-returner`) for the two racing calls, decoupling the mutex-level race under test from the dedup layer's own sequencing.

Both fixes were verified against the unimplemented stub first (still 20/20 failures - the fixes didn't accidentally make anything vacuous) before re-running `attempt-good`, which then passed 20/20 cleanly.

## Step 3 (continued): the naive attempt's own bug initially went uncaught too

With the two fixes above in place, `attempt-naive-client-scoped-dedup` still passed all 20 tests on the first run - including `dedup_is_scoped_per_client_and_resource_not_client_alone`, the test specifically written to catch exactly this bug class. Inspection found the test asserted only `matches!(b, Response::Granted { .. })` on the second (cross-resource) checkout - true whether that response came from actually executing the second checkout or from replaying the first resource's identically-shaped cached grant, since both are the same enum variant. **This test was vacuous against its own named bug.**

Fixed by asserting the *effect*, not just the response shape: after the second checkout call, `service.status("repo/backend")` must report the resource genuinely held by its own caller (`"alice-backend"`) - a claim that can only be true if `do_checkout` actually ran against `repo/backend`'s own state, which a client-scoped dedup replay would have skipped entirely. Re-verified against all three states: still 20/20 on the stub-must-fail check (no regression - N/A here, since a stub failure is total by construction), 20/20 on `attempt-good`, and now **19/20 on `attempt-naive-client-scoped-dedup`**, failing exactly and only the intended test:

```
thread 'dedup_is_scoped_per_client_and_resource_not_client_alone' panicked:
repo/backend must actually be held - a dedup key scoped by client_id alone would have
replayed repo/frontend's cached response without ever executing this checkout against
repo/backend's own state
```

## Step 3 (continued): what clippy did and didn't catch

**`cargo clippy --tests -- -D warnings` is clean on both attempts.** Unlike Module 01's own finding (a `read()`-vs-`read_exact()` bug that `clippy::unused_io_amount` caught instantly, deterministically, on every run), a dedup key missing a struct field has no clippy lint that could plausibly catch it - this is a domain-logic/state-correctness bug, not a language-level footgun. **This confirms the prediction in Module 01's own retro (`runs/2026-08-29-module-01-dry-run/retro.md`)**: the "some fault classes need clippy, not tests" pattern does not generalize to this module's own bug surface. Here, the deterministic test suite is the *only* mechanism that catches anything - which makes the vacuous-test finding above more consequential than it would be if clippy offered a second, independent line of defense the way it did in Module 01.

## Step 4: Score against the rubric

| # | Criterion | attempt-good | attempt-naive-client-scoped-dedup |
|---|---|---|---|
| 1 | `cargo test --test module_02_checkout_service` is green | Pass. 20/20. | **Fails.** 19/20 - `dedup_is_scoped_per_client_and_resource_not_client_alone`. |
| 2 | `cargo clippy --tests -- -D warnings` is clean | Pass. | Pass - clippy is orthogonal to this bug class; see above. |
| 3 | The diff touches only `src/checkout.rs` | Pass. | Pass. |
| 4 | Dedup is keyed by `(client_id, resource)`, never `client_id` alone | Pass. | **Fails**, redundantly with criterion 1 - deliberate, matching Module 01's own precedent for criterion 4 (a learner who somehow gets the test suite to stop exercising this - e.g. by weakening or deleting the specific test - doesn't get a free pass; Coachgremlin checks the property directly). |
| 5 | Expiry is derived from the injected `Clock` on every call, never cached or assumed from a prior check - `status` and `handle` agree independently | Pass. | Pass - orthogonal to this attempt's injected bug. |
| 6 | No background timer, thread, or task advances state on its own - every state change traces to a `handle`/`status` call | Pass. | Pass - orthogonal to this attempt's injected bug. |

**Result: the test suite alone was sufficient to separate the two attempts here - the mirror image of Module 01's finding**, where `cargo clippy` did the separating and `cargo test` passed identically on both. Confirms the two modules' bug surfaces are different in kind, exactly as predicted, and argues against ever assuming one module's tier balance for another without checking.

## Step 5: Confirm or loop

- **attempt-good:** rubric met.
- **attempt-naive-client-scoped-dedup:** rubric not met (gate criterion 1 fails outright - this attempt would not advance).

## Takeaway

Packaged: the deterministic-tier test suite itself (`fixtures/checkout/tests/module_02_checkout_service.rs`), now verified discriminating against a real, plausible mistake rather than merely inspected. The concurrency-test pattern (racing calls under real OS thread scheduling, asserting an invariant across many repeated trials rather than relying on `turmoil`-seeded reproducibility, since a purely in-memory `Mutex`-guarded service has no simulated network for `turmoil` to schedule) is new to this workshop and worth reusing as a template for any future module whose exercise is similarly network-free.

## What this dry run is and isn't evidence of

**Is:** confirmation that Module 02's exercise, test suite, and rubric are real and internally consistent - a correct attempt passes everything, a real, honest wrong attempt fails at least one gate criterion, and the *reason* it fails is understood and documented.

**Is also:** direct, first-time confirmation of Module 01's own retro instruction to check "clippy vs. tests" fresh rather than assume it generalizes - it doesn't, and knowing that in advance would have been guessing, not evidence. Worth continuing to check this per module rather than converging on a rule after only two data points.

**Is also, more pointedly:** two real, load-bearing bugs found in this dry run's own test-authoring process, not in the exercise or its reference solution - a test that would have falsely failed a genuinely correct implementation (the sequence-reuse and mixed-client-id issues), and a test that would have falsely passed a genuinely buggy one (the vacuous `matches!`-only dedup check). Both were only caught by actually running both a correct and an incorrect real implementation through the suite before trusting it - exactly the discipline this dry-run process exists to enforce, and exactly why "the tests look right on inspection" is not a substitute for running them against real code.

**Isn't:** evidence that concurrency bugs in general are hard to test deterministically in this workshop - only that *this* module's exercise has no simulated network to seed against, unlike every other module in the arc; Modules 03+ return to `turmoil`-seeded determinism once real RPC/replication is back in scope.

**Isn't:** a new, independent data point toward Coachgremlin's 3-run Review Trigger (that bar counts distinct workshops, not modules within one) - but it is real, second-time evidence for this workshop's own dry-run discipline catching real problems before a learner ever sees them.
