# Module 02 Rubric: A Single-Node Checkout Service

Shared with the learner before the attempt, per Coachgremlin's own workflow. Criteria are property-phrased (an observable fact about the finished code), not technique-named (the fix itself stated outright).

| # | Criterion | Tier |
|---|---|---|
| 1 | `cargo test --test module_02_checkout_service` is green. | Gate, deterministic |
| 2 | `cargo clippy --tests -- -D warnings` is clean. | Gate, deterministic |
| 3 | The diff touches only `src/checkout.rs` - not `tests/module_02_checkout_service.rs`. | Gate, anti-gaming |
| 4 | Deduplication is keyed by `(client_id, resource)`, never `client_id` alone - a client holding two concurrent leases on different resources must not have one's cached response clobber the other's. | Scored, conceptual |
| 5 | Lease expiry is derived from the injected `Clock` at the moment of *every* call that reports or depends on it (`handle` and `status` alike) - never cached from a prior check, and never assumed still valid because nothing has "happened" recently. | Scored, conceptual |
| 6 | No background timer, thread, or task advances any state on its own - every state change traces back to a specific `handle` (or, for a read, `status`) call. | Scored, conceptual |

## Why criterion 4 is scored, not gated

A real, evidenced finding from this module's own dry run (`runs/2026-08-31-module-02-dry-run/`): a dedup map keyed by `client_id` alone - dropping `resource` from the key - passed `cargo test` 19/20 on first write, because the one test written specifically to catch it asserted only that the second call's response was *some* `Granted` variant, which is true whether that response came from actually executing the second checkout or from replaying the first resource's identically-shaped cached grant. Fixed by asserting the *effect* (the second resource must actually show as held by its own caller afterward), which correctly drops the naive attempt to 19/20. **`cargo clippy` was clean on both attempts** - unlike Module 01's `read()`/`read_exact()` finding, this bug class has no clippy lint that could plausibly express it, so criterion 2 does none of the discriminating work here; criterion 1 (tests green) is the only mechanical gate that catches it. Criterion 4 exists so a learner who somehow weakens or narrows this specific test doesn't get a free pass - Coachgremlin checks the property directly, not just the test suite's current state.

## Why criteria 5 and 6 exist as separate, explicit properties

Both are about the same underlying requirement (no wall-clock reliance, no autonomous background state change) stated as two distinct, independently-checkable facts rather than one vague "handles time correctly" line - a design choice carried over from this module's own learning objectives, which call out both "propose a clock advancement on your own" (there is no autonomous advancement at this module's scope - time only ever moves when the *test* advances it, and the service must correctly re-derive expiry from whatever the clock currently reads) and "no real wall-clock reliance from the start" as separate concerns a learner could satisfy one of without the other (e.g., correctly injecting a `Clock` trait but still caching an `is_expired` boolean at write time instead of re-deriving it on every read).

## A named exception to this workshop's usual determinism norm

Two tests in the provided suite (`only_one_of_two_racing_checkouts_for_the_same_resource_ever_succeeds`, `a_racing_renew_and_return_never_leave_the_resource_in_a_corrupted_state`) are not `turmoil`-seeded and not exactly reproducible seed-to-seed, unlike every other test in this workshop's suites so far. This module's exercise has no simulated network at all - `CheckoutService` is a pure, in-memory, `Mutex`-guarded type - so there is nothing for `turmoil` to schedule. These two tests instead run the same race under real OS thread scheduling across many repeated trials (200 by default) and assert the invariant holds on every one, a standard way to test concurrency correctness without needing exact reproducibility. Their confidence scales with trial count and real scheduler variance, not with a seed number the way every other test in this workshop does - named here explicitly rather than left for a learner to notice (or not) on their own.

## What "explain why" means for criteria 4-6

Not "the test suite passes." The learner should be able to state, in their own words, why a per-client-only dedup key is wrong (it conflates two different resources' independent retry histories into one), why re-deriving expiry on every call matters even though nothing has "happened" recently (a resource nobody has touched since it was granted must still correctly report expired once its lease has lapsed), and why no background timer is used at all (it would be real, autonomous state change this service's single-node design has no way to keep deterministic once it isn't single-node anymore).
