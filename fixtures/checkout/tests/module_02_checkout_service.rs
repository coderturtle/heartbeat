//! Deterministic gate for Module 02: A Single-Node Checkout Service.
//!
//! No network, no turmoil - `CheckoutService` is a pure in-memory type, and
//! everything here calls it directly. Time is supplied by a manually-
//! advanced test clock (never a real wall clock), so every non-concurrent
//! test here is exactly reproducible. The handful of genuinely concurrent
//! tests at the bottom can't be seed-reproducible the way the rest of this
//! workshop's tests are - a `Mutex`-guarded, purely in-memory service has
//! no simulated network for `turmoil` to schedule - so they instead assert
//! an invariant holds across many repeated trials under real OS thread
//! scheduling, a standard way to test concurrency correctness without
//! needing exact reproducibility.

use checkout::checkout::{CheckoutOp, CheckoutService, Clock, DenyReason, Request, Response};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};

#[derive(Clone)]
struct TestClock(Arc<AtomicU64>);

impl TestClock {
    fn new() -> Self {
        Self(Arc::new(AtomicU64::new(0)))
    }

    fn advance(&self, ms: u64) {
        self.0.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

fn req(client_id: &str, resource: &str, sequence: u64, op: CheckoutOp) -> Request {
    Request {
        client_id: client_id.to_string(),
        resource: resource.to_string(),
        sequence,
        op,
    }
}

fn checkout_op(holder: &str, lease_duration_ms: u64) -> CheckoutOp {
    CheckoutOp::Checkout {
        holder: holder.to_string(),
        lease_duration_ms,
    }
}

fn renew_op(holder: &str, generation: u64, lease_duration_ms: u64) -> CheckoutOp {
    CheckoutOp::Renew {
        holder: holder.to_string(),
        generation,
        lease_duration_ms,
    }
}

fn return_op(holder: &str, generation: u64) -> CheckoutOp {
    CheckoutOp::Return {
        holder: holder.to_string(),
        generation,
    }
}

fn granted_generation(response: &Response) -> u64 {
    match response {
        Response::Granted { generation, .. } => *generation,
        other => panic!("expected Granted, got {other:?}"),
    }
}

// ─── Exclusivity and basic lifecycle ────────────────────────────────────

#[test]
fn checkout_grants_exclusive_access() {
    let service = CheckoutService::new(TestClock::new());

    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
    assert!(matches!(granted, Response::Granted { .. }), "first checkout should be granted, got {granted:?}");

    let denied = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)));
    assert_eq!(denied, Response::Denied(DenyReason::HeldByOther));
}

#[test]
fn a_lease_survives_until_its_duration_elapses_then_the_resource_is_reissuable() {
    let clock = TestClock::new();
    let service = CheckoutService::new(clock.clone());

    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)));

    clock.advance(999);
    let still_denied = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)));
    assert_eq!(still_denied, Response::Denied(DenyReason::HeldByOther), "the lease has not lapsed yet at t=999");

    clock.advance(2);
    // A genuinely new attempt, not a retry of the one above - a real client
    // wouldn't reuse sequence 1 for a second, later checkout attempt; doing
    // so here would just replay the cached denial from t=999 verbatim,
    // which is correct dedup behavior, not a signal about lease expiry.
    let granted = service.handle(&req("bob", "repo/main", 2, checkout_op("bob-session", 60_000)));
    assert!(matches!(granted, Response::Granted { .. }), "the lease should have lapsed by t=1001");
}

#[test]
fn generation_increases_on_every_new_grant_and_never_resets() {
    let clock = TestClock::new();
    let service = CheckoutService::new(clock.clone());

    let first = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)));
    let first_gen = granted_generation(&first);

    clock.advance(2_000);
    let second = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 1_000)));
    let second_gen = granted_generation(&second);
    assert!(second_gen > first_gen, "seed {first_gen} -> {second_gen}: generation must strictly increase across a free resource");

    let returned = service.handle(&req("bob", "repo/main", 2, return_op("bob-session", second_gen)));
    assert_eq!(returned, Response::Returned);

    let third = service.handle(&req("carol", "repo/main", 1, checkout_op("carol-session", 1_000)));
    let third_gen = granted_generation(&third);
    assert!(third_gen > second_gen, "a returned resource's generation counter must not reset");
}

#[test]
fn generation_counter_survives_even_with_no_active_lease() {
    let clock = TestClock::new();
    let service = CheckoutService::new(clock);

    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)));
    let generation = granted_generation(&granted);
    service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)));

    let status = service.status("repo/main");
    assert!(status.held.is_none(), "resource should not be reported held after return");
    assert_eq!(
        status.last_issued_generation, generation,
        "the highest-ever-issued generation must persist even while nothing currently holds the resource"
    );
}

// ─── Renew ───────────────────────────────────────────────────────────────

#[test]
fn renew_extends_the_lease_when_generation_matches() {
    let clock = TestClock::new();
    let service = CheckoutService::new(clock.clone());

    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)));
    let generation = granted_generation(&granted);

    clock.advance(900);
    let renewed = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", generation, 1_000)));
    let Response::Granted { generation: renewed_gen, expires_at_ms } = renewed else {
        panic!("expected Granted from a valid renew, got {renewed:?}");
    };
    assert_eq!(renewed_gen, generation, "renew must not change the generation");
    assert_eq!(expires_at_ms, 900 + 1_000);

    // Without the renewal, the lease from t=0 would have lapsed by t=1000.
    clock.advance(200);
    let still_held = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)));
    assert_eq!(still_held, Response::Denied(DenyReason::HeldByOther), "the renewal should have extended the lease past t=1100");
}

#[test]
fn renew_with_stale_generation_is_denied() {
    let service = CheckoutService::new(TestClock::new());
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
    let generation = granted_generation(&granted);

    let denied = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", generation + 1, 60_000)));
    assert_eq!(denied, Response::Denied(DenyReason::StaleGeneration));
}

#[test]
fn renew_from_the_wrong_holder_is_denied_even_with_the_right_generation() {
    let service = CheckoutService::new(TestClock::new());
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
    let generation = granted_generation(&granted);

    let denied = service.handle(&req("mallory", "repo/main", 1, renew_op("not-alice-session", generation, 60_000)));
    assert_eq!(denied, Response::Denied(DenyReason::StaleGeneration));
}

#[test]
fn renew_after_expiry_is_denied_as_not_held_not_stale_generation() {
    let clock = TestClock::new();
    let service = CheckoutService::new(clock.clone());

    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)));
    let generation = granted_generation(&granted);

    clock.advance(1_001);
    let denied = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", generation, 60_000)));
    assert_eq!(denied, Response::Denied(DenyReason::NotHeld), "a lapsed lease can't be renewed back to life, even with the correct generation");
}

// ─── Return ──────────────────────────────────────────────────────────────

#[test]
fn return_releases_the_resource_for_a_new_grant() {
    let service = CheckoutService::new(TestClock::new());
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
    let generation = granted_generation(&granted);

    let returned = service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)));
    assert_eq!(returned, Response::Returned);

    let granted_again = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)));
    assert!(matches!(granted_again, Response::Granted { .. }));
}

#[test]
fn return_with_wrong_generation_or_holder_is_denied() {
    let service = CheckoutService::new(TestClock::new());
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
    let generation = granted_generation(&granted);

    let wrong_generation = service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation + 1)));
    assert_eq!(wrong_generation, Response::Denied(DenyReason::StaleGeneration));

    let wrong_holder = service.handle(&req("mallory", "repo/main", 1, return_op("not-alice-session", generation)));
    assert_eq!(wrong_holder, Response::Denied(DenyReason::StaleGeneration));
}

#[test]
fn return_on_a_resource_with_no_active_lease_is_not_held() {
    let service = CheckoutService::new(TestClock::new());
    let denied = service.handle(&req("alice", "repo/main", 1, return_op("alice-session", 1)));
    assert_eq!(denied, Response::Denied(DenyReason::NotHeld));
}

// ─── Lease duration cap ──────────────────────────────────────────────────

#[test]
fn requested_lease_duration_is_capped_on_checkout_and_renew() {
    use checkout::checkout::MAX_LEASE_DURATION_MS;

    let service = CheckoutService::new(TestClock::new());
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", MAX_LEASE_DURATION_MS * 10)));
    let Response::Granted { generation, expires_at_ms } = granted else {
        panic!("expected Granted, got {granted:?}");
    };
    assert_eq!(expires_at_ms, MAX_LEASE_DURATION_MS, "an oversized initial lease duration must be capped, not rejected");

    let renewed = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", generation, MAX_LEASE_DURATION_MS * 10)));
    let Response::Granted { expires_at_ms: renewed_expiry, .. } = renewed else {
        panic!("expected Granted, got {renewed:?}");
    };
    assert_eq!(renewed_expiry, MAX_LEASE_DURATION_MS, "an oversized renewal duration must also be capped, not just the initial grant");
}

// ─── Deduplication ───────────────────────────────────────────────────────

#[test]
fn a_retried_sequence_replays_the_cached_response_without_re_executing() {
    let clock = TestClock::new();
    let service = CheckoutService::new(clock.clone());

    let first = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));

    // If the retry re-executed, this would now see the resource already
    // held by "alice-session" and deny it - the replayed response must
    // still be the original Granted.
    let retried = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
    assert_eq!(first, retried, "an exact sequence retry must replay the original response verbatim");
}

#[test]
fn a_cached_denial_replays_identically_on_retry() {
    let service = CheckoutService::new(TestClock::new());
    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));

    let denied = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)));
    assert_eq!(denied, Response::Denied(DenyReason::HeldByOther));

    let retried = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)));
    assert_eq!(retried, Response::Denied(DenyReason::HeldByOther), "a cached denial must replay identically too, not flip on retry");
}

#[test]
fn dedup_is_scoped_per_client_and_resource_not_client_alone() {
    let service = CheckoutService::new(TestClock::new());

    let frontend = service.handle(&req("alice", "repo/frontend", 1, checkout_op("alice-frontend", 60_000)));
    assert!(matches!(frontend, Response::Granted { .. }));

    // Same client, same sequence number, but a *different* resource - a
    // dedup key scoped by client_id alone would find seq 1 already cached
    // (from the frontend call above) and replay that cached response
    // instead of actually executing this checkout - `matches!` alone can't
    // tell the difference here, since both outcomes are some `Granted`
    // variant. The real check is below: did repo/backend's own state
    // actually change.
    let backend = service.handle(&req("alice", "repo/backend", 1, checkout_op("alice-backend", 60_000)));
    assert!(matches!(backend, Response::Granted { .. }));

    let backend_status = service.status("repo/backend");
    let held = backend_status.held.expect(
        "repo/backend must actually be held - a dedup key scoped by client_id alone would have replayed repo/frontend's cached response \
         without ever executing this checkout against repo/backend's own state",
    );
    assert_eq!(held.holder, "alice-backend", "repo/backend must be held by its own caller, not a stale cross-resource replay");

    // The frontend's own entry must still be independently replayable.
    let frontend_replayed = service.handle(&req("alice", "repo/frontend", 1, checkout_op("alice-frontend", 60_000)));
    assert_eq!(frontend, frontend_replayed, "seq 1 against repo/frontend must still replay repo/frontend's own cached response");
}

#[test]
fn a_sequence_older_than_the_highest_seen_is_rejected_not_re_executed() {
    let service = CheckoutService::new(TestClock::new());
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
    let generation = granted_generation(&granted);
    service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)));

    // Sequence 1 already happened and was superseded by sequence 2 - a
    // late-arriving retry of sequence 1 must not be treated as a brand new
    // request against the now-different world state.
    let stale = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
    assert_eq!(stale, Response::Denied(DenyReason::StaleSequence));
}

// ─── Canonicalization ────────────────────────────────────────────────────

#[test]
fn resource_identifiers_are_canonicalized_at_the_boundary() {
    let service = CheckoutService::new(TestClock::new());
    service.handle(&req("alice", "  repo/main  ", 1, checkout_op("alice-session", 60_000)));

    let denied = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)));
    assert_eq!(denied, Response::Denied(DenyReason::HeldByOther), "leading/trailing whitespace must not create a second identity for the same resource");

    let status = service.status(" repo/main ");
    assert!(status.held.is_some(), "status must canonicalize its own argument the same way handle() does");
}

// ─── Status ──────────────────────────────────────────────────────────────

#[test]
fn status_independently_reports_expiry_without_any_intervening_write() {
    let clock = TestClock::new();
    let service = CheckoutService::new(clock.clone());
    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)));

    clock.advance(1_001);
    let status = service.status("repo/main");
    assert!(status.held.is_none(), "status alone (no checkout/renew/return in between) must re-derive expiry from the current clock, not trust stale stored state");
}

// ─── Concurrency ─────────────────────────────────────────────────────────
//
// These can't be turmoil-seeded (no simulated network exists for a purely
// in-memory service to schedule), so each runs the same race many times
// under real OS thread scheduling and asserts the invariant holds on every
// trial - a standard way to test concurrency correctness without needing
// exact reproducibility.

const CONCURRENCY_TRIALS: usize = 200;

#[test]
fn only_one_of_two_racing_checkouts_for_the_same_resource_ever_succeeds() {
    for trial in 0..CONCURRENCY_TRIALS {
        let service = Arc::new(CheckoutService::new(TestClock::new()));
        let barrier = Arc::new(Barrier::new(2));

        let run = |holder: &'static str| {
            let service = Arc::clone(&service);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                service.handle(&req(holder, "repo/main", 1, checkout_op(holder, 60_000)))
            })
        };

        let a = run("alice-session");
        let b = run("bob-session");
        let (a, b) = (a.join().unwrap(), b.join().unwrap());

        let grants = [&a, &b].into_iter().filter(|r| matches!(r, Response::Granted { .. })).count();
        assert_eq!(grants, 1, "trial {trial}: exactly one of two concurrent checkouts for the same free resource must succeed, got a={a:?} b={b:?}");
    }
}

#[test]
fn a_racing_renew_and_return_never_leave_the_resource_in_a_corrupted_state() {
    for trial in 0..CONCURRENCY_TRIALS {
        let clock = TestClock::new();
        let service = Arc::new(CheckoutService::new(clock));
        let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)));
        let generation = granted_generation(&granted);

        let barrier = Arc::new(Barrier::new(2));
        let renew_service = Arc::clone(&service);
        let renew_barrier = Arc::clone(&barrier);
        // Different client_ids for the two racing calls, deliberately - a
        // single client_id's sequence numbers express strict retry order
        // for that one client, not "these two calls are concurrent"; two
        // different sequence numbers under the same client_id would just
        // engage the dedup path's own (correct) stale-sequence rejection
        // instead of exercising the resource-state race this test wants.
        let renew_handle = std::thread::spawn(move || {
            renew_barrier.wait();
            renew_service.handle(&req("alice-renewer", "repo/main", 1, renew_op("alice-session", generation, 60_000)))
        });
        let return_service = Arc::clone(&service);
        let return_barrier = Arc::clone(&barrier);
        let return_handle = std::thread::spawn(move || {
            return_barrier.wait();
            return_service.handle(&req("alice-returner", "repo/main", 1, return_op("alice-session", generation)))
        });

        let renew_result = renew_handle.join().unwrap();
        let return_result = return_handle.join().unwrap();

        // Whichever order the two calls actually serialized in, both must
        // be individually well-formed responses (never a panic, never a
        // torn/inconsistent Response), and the resource's final state must
        // be self-consistent with whichever one applied last.
        assert!(
            matches!(renew_result, Response::Granted { .. } | Response::Denied(DenyReason::NotHeld)),
            "trial {trial}: renew raced against return produced an unexpected response: {renew_result:?}"
        );
        assert_eq!(return_result, Response::Returned, "trial {trial}: return must always succeed regardless of a concurrent renew attempt");

        let status = service.status("repo/main");
        assert!(status.held.is_none(), "trial {trial}: after a return commits, the resource must be reported free regardless of a concurrent renew's outcome");
    }
}
