//! Deterministic gate for Module 02: A Single-Node Checkout Service.
//!
//! No network, no turmoil - `CheckoutService` is a pure in-memory type, and
//! everything here calls it directly, passing an explicit `now_ms` at every
//! call site rather than relying on any clock the service owns. Every
//! non-concurrent test here is exactly reproducible for that reason. The
//! handful of genuinely concurrent tests at the bottom can't be
//! seed-reproducible the way the rest of this workshop's tests are - a
//! `Mutex`-guarded, purely in-memory service has no simulated network for
//! `turmoil` to schedule - so they instead assert an invariant holds across
//! many repeated trials under real OS thread scheduling, a standard way to
//! test concurrency correctness without needing exact reproducibility.

use checkout::checkout::{CheckoutOp, CheckoutService, DenyReason, Request, Response};
use std::sync::{Arc, Barrier};

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
    let service = CheckoutService::new();

    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    assert!(matches!(granted, Response::Granted { .. }), "first checkout should be granted, got {granted:?}");

    let denied = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 0);
    assert_eq!(denied, Response::Denied(DenyReason::Held));
}

#[test]
fn a_lease_survives_until_its_duration_elapses_then_the_resource_is_reissuable() {
    let service = CheckoutService::new();

    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);

    let still_denied = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 999);
    assert_eq!(still_denied, Response::Denied(DenyReason::Held), "the lease has not lapsed yet at t=999");

    // A genuinely new attempt, not a retry of the one above - a real client
    // wouldn't reuse sequence 1 for a second, later checkout attempt; doing
    // so here would just replay the cached denial from t=999 verbatim,
    // which is correct dedup behavior, not a signal about lease expiry.
    let granted = service.handle(&req("bob", "repo/main", 2, checkout_op("bob-session", 60_000)), 1_001);
    assert!(matches!(granted, Response::Granted { .. }), "the lease should have lapsed by t=1001");
}

#[test]
fn expiry_boundary_is_treated_as_expired_not_still_held() {
    let service = CheckoutService::new();
    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);

    // expires_at_ms == 1_000 exactly - `handle` and `status` must agree
    // that this is already expired, not still valid for one more instant.
    let granted_at_boundary = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 1_000);
    assert!(
        matches!(granted_at_boundary, Response::Granted { .. }),
        "a lease with expires_at_ms == now_ms must be treated as already expired, got {granted_at_boundary:?}"
    );

    let service2 = CheckoutService::new();
    service2.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);
    let status_at_boundary = service2.status("repo/main", 1_000);
    assert!(status_at_boundary.held.is_none(), "status must independently agree that expires_at_ms == now_ms means expired");
}

#[test]
fn generation_increases_on_every_new_grant_and_never_resets() {
    let service = CheckoutService::new();

    let first = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);
    let first_gen = granted_generation(&first);

    let second = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 1_000)), 2_000);
    let second_gen = granted_generation(&second);
    assert!(second_gen > first_gen, "{first_gen} -> {second_gen}: generation must strictly increase across a free resource");

    let returned = service.handle(&req("bob", "repo/main", 2, return_op("bob-session", second_gen)), 2_000);
    assert_eq!(returned, Response::Returned);

    let third = service.handle(&req("carol", "repo/main", 1, checkout_op("carol-session", 1_000)), 2_000);
    let third_gen = granted_generation(&third);
    assert!(third_gen > second_gen, "a returned resource's generation counter must not reset");
}

#[test]
fn generation_counter_survives_even_with_no_active_lease() {
    let service = CheckoutService::new();

    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);
    let generation = granted_generation(&granted);
    service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)), 0);

    let status = service.status("repo/main", 0);
    assert!(status.held.is_none(), "resource should not be reported held after return");
    assert_eq!(
        status.last_issued_generation, generation,
        "the highest-ever-issued generation must persist even while nothing currently holds the resource"
    );
}

#[test]
fn generation_counters_are_independent_per_resource() {
    let service = CheckoutService::new();

    let a = granted_generation(&service.handle(&req("alice", "repo/a", 1, checkout_op("alice-a", 1_000)), 0));
    // repo/a issues several generations before repo/b ever sees one.
    service.handle(&req("alice", "repo/a", 2, return_op("alice-a", a)), 0);
    service.handle(&req("alice", "repo/a", 3, checkout_op("alice-a", 1_000)), 0);

    let b = granted_generation(&service.handle(&req("bob", "repo/b", 1, checkout_op("bob-b", 1_000)), 0));
    assert_eq!(b, 1, "repo/b's own first-ever grant must be generation 1, independent of how many generations repo/a has already issued");
}

// ─── Renew ───────────────────────────────────────────────────────────────

#[test]
fn renew_extends_the_lease_when_generation_matches() {
    let service = CheckoutService::new();

    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);
    let generation = granted_generation(&granted);

    let renewed = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", generation, 1_000)), 900);
    let Response::Granted { generation: renewed_gen, expires_at_ms } = renewed else {
        panic!("expected Granted from a valid renew, got {renewed:?}");
    };
    assert_eq!(renewed_gen, generation, "renew must not change the generation");
    assert_eq!(expires_at_ms, 900 + 1_000);

    // Without the renewal, the lease from t=0 would have lapsed by t=1000.
    let still_held = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 1_100);
    assert_eq!(still_held, Response::Denied(DenyReason::Held), "the renewal should have extended the lease past t=1100");
}

#[test]
fn renew_with_stale_generation_is_denied() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    let generation = granted_generation(&granted);

    let denied = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", generation + 1, 60_000)), 0);
    assert_eq!(denied, Response::Denied(DenyReason::StaleGeneration));
}

#[test]
fn renew_from_the_wrong_holder_is_denied_even_with_the_right_generation() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    let generation = granted_generation(&granted);

    let denied = service.handle(&req("mallory", "repo/main", 1, renew_op("not-alice-session", generation, 60_000)), 0);
    assert_eq!(denied, Response::Denied(DenyReason::StaleGeneration));

    // The real check: a denied renew must not have mutated the lease at
    // all - a mutate-then-validate implementation could still extend
    // expires_at_ms before discovering the holder doesn't match.
    let status = service.status("repo/main", 0);
    let held = status.held.expect("the resource must still be held after a denied renew");
    assert_eq!(held.expires_at_ms, 60_000, "a denied renew must not have extended the lease at all");
    assert_eq!(held.holder, "alice-session", "a denied renew must not have changed the holder");
}

#[test]
fn renew_after_expiry_is_denied_as_not_held_not_stale_generation() {
    let service = CheckoutService::new();

    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);
    let generation = granted_generation(&granted);

    let denied = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", generation, 60_000)), 1_001);
    assert_eq!(denied, Response::Denied(DenyReason::NotHeld), "a lapsed lease can't be renewed back to life, even with the correct generation");
}

#[test]
fn renew_after_the_resource_has_been_returned_is_not_held() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    let generation = granted_generation(&granted);
    service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)), 0);

    // Deterministic complement to the concurrent renew-vs-return test below:
    // once a resource has genuinely been returned, a renew attempt with the
    // exact generation that used to be valid must be treated the same way
    // as an expired lease - NotHeld, not StaleGeneration (the generation
    // itself isn't wrong; the lease it belonged to is simply gone).
    let denied = service.handle(&req("alice", "repo/main", 3, renew_op("alice-session", generation, 60_000)), 0);
    assert_eq!(denied, Response::Denied(DenyReason::NotHeld));
}

// ─── Return ──────────────────────────────────────────────────────────────

#[test]
fn return_releases_the_resource_for_a_new_grant() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    let generation = granted_generation(&granted);

    let returned = service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)), 0);
    assert_eq!(returned, Response::Returned);

    let granted_again = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 0);
    assert!(matches!(granted_again, Response::Granted { .. }));
}

#[test]
fn return_with_wrong_generation_or_holder_is_denied() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    let generation = granted_generation(&granted);

    let wrong_generation = service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation + 1)), 0);
    assert_eq!(wrong_generation, Response::Denied(DenyReason::StaleGeneration));

    let wrong_holder = service.handle(&req("mallory", "repo/main", 1, return_op("not-alice-session", generation)), 0);
    assert_eq!(wrong_holder, Response::Denied(DenyReason::StaleGeneration));

    // Neither denied return may have actually released the resource.
    let status = service.status("repo/main", 0);
    let held = status.held.expect("the resource must still be held after two denied returns");
    assert_eq!(held.holder, "alice-session");
    assert_eq!(held.generation, generation);
}

#[test]
fn return_on_a_resource_with_no_active_lease_is_not_held() {
    let service = CheckoutService::new();
    let denied = service.handle(&req("alice", "repo/main", 1, return_op("alice-session", 1)), 0);
    assert_eq!(denied, Response::Denied(DenyReason::NotHeld));
}

#[test]
fn return_on_an_expired_lease_is_not_held() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);
    let generation = granted_generation(&granted);

    // The generation is still the correct one - it's the lease itself that
    // has lapsed. Return must treat this the same way Renew does: NotHeld,
    // not a silent success (a dead lease can't be explicitly returned back
    // to life any more than it can be renewed).
    let denied = service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)), 1_001);
    assert_eq!(denied, Response::Denied(DenyReason::NotHeld));
}

// ─── Fencing ─────────────────────────────────────────────────────────────

#[test]
fn a_delayed_command_against_a_superseded_generation_is_denied_not_honored() {
    let service = CheckoutService::new();

    // Alice holds generation 1, which then lapses.
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);
    let alice_generation = granted_generation(&granted);

    // Bob checks out the now-free resource, becoming generation 2.
    let bob_granted = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 1_001);
    let bob_generation = granted_generation(&bob_granted);
    assert!(bob_generation > alice_generation);

    // Alice's own delayed Renew, carrying her now-superseded generation,
    // finally arrives - this must never be allowed to affect the lease
    // Bob now holds, fencing's entire reason for existing.
    let alice_delayed_renew = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", alice_generation, 60_000)), 1_500);
    assert_eq!(alice_delayed_renew, Response::Denied(DenyReason::StaleGeneration));

    let status = service.status("repo/main", 1_500);
    let held = status.held.expect("bob's lease must still be held");
    assert_eq!(held.holder, "bob-session", "alice's delayed renew must not have disturbed bob's lease");
    assert_eq!(held.generation, bob_generation);
}

// ─── Same-holder re-checkout ─────────────────────────────────────────────

#[test]
fn same_holder_recheckout_with_a_new_sequence_is_still_denied_held_by_other() {
    // A caller who wants to keep their own lease alive uses Renew, not a
    // fresh Checkout - this service does not special-case "the caller
    // already holds this" as anything other than "still held." Idempotency
    // for a *retried* checkout is what dedup (sequence replay) exists for;
    // a genuinely new sequence is a genuinely new request, decided purely
    // by whether the resource is currently held, holder identity aside.
    let service = CheckoutService::new();
    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);

    let second_attempt = service.handle(&req("alice", "repo/main", 2, checkout_op("alice-session", 60_000)), 0);
    assert_eq!(second_attempt, Response::Denied(DenyReason::Held));
}

// ─── Lease duration cap ──────────────────────────────────────────────────

#[test]
fn requested_lease_duration_is_capped_on_checkout_and_renew() {
    use checkout::checkout::MAX_LEASE_DURATION_MS;

    let service = CheckoutService::new();
    let now = 1_000_000;
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", MAX_LEASE_DURATION_MS * 10)), now);
    let Response::Granted { generation, expires_at_ms } = granted else {
        panic!("expected Granted, got {granted:?}");
    };
    // Pins "cap the requested duration, then add to now" against the
    // plausible-but-wrong "clamp the resulting absolute timestamp" - the
    // two formulas only agree at now=0, which every other test in this
    // suite uses. At a nonzero now, clamping produces an
    // already-expired-on-arrival expires_at_ms; capping produces a lease
    // that's actually usable.
    assert_eq!(expires_at_ms, now + MAX_LEASE_DURATION_MS, "the requested duration must be capped, then added to now - not the resulting timestamp clamped to the constant's own raw value");

    let renewed = service.handle(&req("alice", "repo/main", 2, renew_op("alice-session", generation, MAX_LEASE_DURATION_MS * 10)), now);
    let Response::Granted { expires_at_ms: renewed_expiry, .. } = renewed else {
        panic!("expected Granted, got {renewed:?}");
    };
    assert_eq!(renewed_expiry, now + MAX_LEASE_DURATION_MS, "an oversized renewal duration must also be capped the same way, not just the initial grant");
}

#[test]
fn max_lease_duration_is_unchanged_from_its_shipped_value() {
    use checkout::checkout::MAX_LEASE_DURATION_MS;
    // Pins the exact constant, not just "some cap exists" - every other
    // test in this suite is satisfied by a much smaller value (traced: the
    // binding floor from the rest of the suite alone is 60_000, one
    // fifth of the real cap), so this is the one thing standing between
    // property 10's stated maximum and a learner quietly shrinking it.
    assert_eq!(MAX_LEASE_DURATION_MS, 5 * 60 * 1000);
}

// ─── Serialization ───────────────────────────────────────────────────────

#[test]
fn every_public_type_actually_serializes() {
    // Not exercised by any other test - a learner could delete every
    // Serialize/Deserialize derive from src/checkout.rs and still pass
    // every other test in this suite. These types cross a Raft log in a
    // later module; a type that can't round-trip through serde would force
    // a change to this already-graded file at that point.
    use checkout::checkout::{HeldInfo, StatusInfo};

    let request = req("alice", "repo/main", 1, checkout_op("alice-session", 60_000));
    let round_tripped: Request = serde_json::from_str(&serde_json::to_string(&request).unwrap()).unwrap();
    assert_eq!(request, round_tripped);

    let response = Response::Denied(DenyReason::StaleGeneration);
    let round_tripped: Response = serde_json::from_str(&serde_json::to_string(&response).unwrap()).unwrap();
    assert_eq!(response, round_tripped);

    let status = StatusInfo {
        resource: "repo/main".to_string(),
        held: Some(HeldInfo {
            holder: "alice-session".to_string(),
            generation: 1,
            expires_at_ms: 60_000,
        }),
        last_issued_generation: 1,
    };
    let round_tripped: StatusInfo = serde_json::from_str(&serde_json::to_string(&status).unwrap()).unwrap();
    assert_eq!(status, round_tripped);
}

// ─── Deduplication ───────────────────────────────────────────────────────

#[test]
fn a_retried_sequence_replays_the_cached_response_without_re_executing() {
    let service = CheckoutService::new();

    let first = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);

    // Advance time past when the original lease would have lapsed, and
    // retry the *exact* same request. If this re-executed instead of
    // replaying, it would come back as a fresh Granted with a different
    // expires_at_ms (1_500 + 1_000, not the original 0 + 1_000) - an
    // idempotent-re-execution implementation and a genuinely-caching one
    // are indistinguishable unless the world has actually changed in
    // between.
    let retried = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 1_500);
    assert_eq!(first, retried, "an exact sequence retry must replay the original cached response verbatim, not re-execute against the new now_ms");
}

#[test]
fn a_cached_denial_replays_identically_on_retry() {
    let service = CheckoutService::new();
    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);

    let denied = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 0);
    assert_eq!(denied, Response::Denied(DenyReason::Held));

    // Advance time past when alice's lease actually lapses, then retry
    // bob's exact same request. If this re-executed, it would now succeed
    // (the resource is genuinely free) - the cached denial must still
    // replay identically, exactly the trap this workshop's own design
    // process named ("a retried denial could wrongly flip to granted").
    let retried = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 2_000);
    assert_eq!(retried, Response::Denied(DenyReason::Held), "a cached denial must replay identically even after the world has changed enough that re-executing it would succeed");
}

#[test]
fn a_cached_return_replays_identically_on_retry() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    let generation = granted_generation(&granted);

    let returned = service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)), 0);
    assert_eq!(returned, Response::Returned);

    // The resource is now genuinely free - if this retry re-executed
    // instead of replaying, do_return would find no active lease at all
    // and answer Denied(NotHeld), not the original Returned. This is the
    // third leg of "granted, denied, or returned - verbatim on retry";
    // the other two are covered above, but neither exercises this path.
    let retried = service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)), 5_000);
    assert_eq!(retried, Response::Returned, "a cached Returned outcome must replay identically, not re-execute against a resource that's now genuinely free");
}

#[test]
fn dedup_is_scoped_per_client_and_resource_not_client_alone() {
    let service = CheckoutService::new();

    let frontend = service.handle(&req("alice", "repo/frontend", 1, checkout_op("alice-frontend", 60_000)), 0);
    assert!(matches!(frontend, Response::Granted { .. }));

    // Same client, same sequence number, but a *different* resource - a
    // dedup key scoped by client_id alone would find seq 1 already cached
    // (from the frontend call above) and replay that cached response
    // instead of actually executing this checkout - `matches!` alone can't
    // tell the difference here, since both outcomes are some `Granted`
    // variant. The real check is below: did repo/backend's own state
    // actually change.
    let backend = service.handle(&req("alice", "repo/backend", 1, checkout_op("alice-backend", 60_000)), 0);
    assert!(matches!(backend, Response::Granted { .. }));

    let backend_status = service.status("repo/backend", 0);
    let held = backend_status.held.expect(
        "repo/backend must actually be held - a dedup key scoped by client_id alone would have replayed repo/frontend's cached response \
         without ever executing this checkout against repo/backend's own state",
    );
    assert_eq!(held.holder, "alice-backend", "repo/backend must be held by its own caller, not a stale cross-resource replay");

    // The frontend's own entry must still be independently replayable.
    let frontend_replayed = service.handle(&req("alice", "repo/frontend", 1, checkout_op("alice-frontend", 60_000)), 0);
    assert_eq!(frontend, frontend_replayed, "seq 1 against repo/frontend must still replay repo/frontend's own cached response");
}

#[test]
fn a_sequence_older_than_the_highest_seen_is_rejected_not_re_executed() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    let generation = granted_generation(&granted);
    service.handle(&req("alice", "repo/main", 2, return_op("alice-session", generation)), 0);

    // Sequence 1 already happened and was superseded by sequence 2 - a
    // late-arriving retry of sequence 1 must not be treated as a brand new
    // request against the now-different world state.
    let stale = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    assert_eq!(stale, Response::Denied(DenyReason::StaleSequence));
}

// ─── Canonicalization ────────────────────────────────────────────────────

#[test]
fn canonicalize_trims_surrounding_whitespace() {
    assert_eq!(CheckoutService::canonicalize("  repo/main  "), "repo/main");
    assert_eq!(CheckoutService::canonicalize("repo/main"), "repo/main");
}

#[test]
fn resource_identifiers_are_canonicalized_at_the_boundary() {
    let service = CheckoutService::new();
    service.handle(&req("alice", "  repo/main  ", 1, checkout_op("alice-session", 60_000)), 0);

    let denied = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 60_000)), 0);
    assert_eq!(denied, Response::Denied(DenyReason::Held), "leading/trailing whitespace must not create a second identity for the same resource");

    let status = service.status(" repo/main ", 0);
    assert!(status.held.is_some(), "status must canonicalize its own argument the same way handle() does");
    assert_eq!(status.resource, "repo/main", "StatusInfo::resource must report the canonical form, not the raw query string");
}

#[test]
fn dedup_key_is_also_canonicalized_not_just_the_exclusivity_check() {
    let service = CheckoutService::new();
    let original = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);

    // The exact same client and sequence, but the resource string is
    // spelled with different surrounding whitespace than the original
    // call - a real retry could plausibly arrive this way (e.g. a client
    // library that re-normalizes input differently on a retry path). If
    // the dedup key used the raw, uncanonicalized resource string, this
    // would be treated as a brand new request against a different
    // "resource" and would either re-execute (denying itself, since the
    // canonical resource is genuinely already held) or - worse - land in
    // its own independent, ungoverned sequence-number space.
    let retried = service.handle(&req("alice", " repo/main ", 1, checkout_op("alice-session", 60_000)), 0);
    assert_eq!(original, retried, "the dedup key must use the canonical resource, so a retry spelled with different whitespace still replays the original cached response");
}

#[test]
fn dedup_key_canonicalizes_client_id_too_not_just_resource() {
    let service = CheckoutService::new();
    let original = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);

    // Same reasoning as the resource half of the dedup key, applied to the
    // client_id half: a retry whose client_id happens to be spelled with
    // different surrounding whitespace must still land in the same
    // (client_id, resource) dedup entry, not a separate, ungoverned one.
    let retried = service.handle(&req(" alice ", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
    assert_eq!(original, retried, "the dedup key must canonicalize client_id as well as resource");
}

// ─── Status purity ───────────────────────────────────────────────────────

#[test]
fn status_independently_reports_expiry_without_any_intervening_write() {
    let service = CheckoutService::new();
    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);

    let status = service.status("repo/main", 1_001);
    assert!(status.held.is_none(), "status alone (no checkout/renew/return in between) must re-derive expiry from now_ms, not trust stale stored state");
}

#[test]
fn status_never_mutates_or_evicts_state() {
    let service = CheckoutService::new();
    let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 0);
    let generation = granted_generation(&granted);

    // Query status well after the lease has lapsed - a plausible but wrong
    // implementation might lazily evict the resource's entry here (a
    // natural place to "clean up" an expired lease on read). If it did,
    // the resource's generation counter would be destroyed along with it.
    let expired_status = service.status("repo/main", 5_000);
    assert!(expired_status.held.is_none());

    let next = service.handle(&req("bob", "repo/main", 1, checkout_op("bob-session", 1_000)), 5_000);
    let next_generation = granted_generation(&next);
    assert!(
        next_generation > generation,
        "a status() call against an already-expired resource must not have evicted its generation counter - the next real Checkout must still continue from it, not restart at 1"
    );

    // The dedup entry must have survived too, not just the resource's own
    // generation counter - a status() call has no business touching either.
    let alice_retried = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 1_000)), 5_000);
    assert_eq!(granted, alice_retried, "status() must not have evicted alice's own dedup entry either");
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
        let service = Arc::new(CheckoutService::new());
        let barrier = Arc::new(Barrier::new(2));

        let run = |holder: &'static str| {
            let service = Arc::clone(&service);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                service.handle(&req(holder, "repo/main", 1, checkout_op(holder, 60_000)), 0)
            })
        };

        let a = run("alice-session");
        let b = run("bob-session");
        let (a, b) = (a.join().unwrap(), b.join().unwrap());

        let grants = [&a, &b].into_iter().filter(|r| matches!(r, Response::Granted { .. })).count();
        assert_eq!(grants, 1, "trial {trial}: exactly one of two concurrent checkouts for the same free resource must succeed, got a={a:?} b={b:?}");

        // The real check: whichever response actually was Granted must
        // match who the service now believes holds the resource - a
        // response-only check can't catch a version where both requests
        // were internally granted and the second silently overwrote the
        // first's lease state before returning Denied.
        let winner = if matches!(a, Response::Granted { .. }) { "alice-session" } else { "bob-session" };
        let status = service.status("repo/main", 0);
        let held = status.held.unwrap_or_else(|| panic!("trial {trial}: the resource must be held by whichever call actually won"));
        assert_eq!(held.holder, winner, "trial {trial}: the service's own recorded holder must match whichever response actually reported Granted");
    }
}

#[test]
fn a_racing_renew_and_return_never_leave_the_resource_in_a_corrupted_state() {
    for trial in 0..CONCURRENCY_TRIALS {
        let service = Arc::new(CheckoutService::new());
        let granted = service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0);
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
            renew_service.handle(&req("alice-renewer", "repo/main", 1, renew_op("alice-session", generation, 60_000)), 0)
        });
        let return_service = Arc::clone(&service);
        let return_barrier = Arc::clone(&barrier);
        let return_handle = std::thread::spawn(move || {
            return_barrier.wait();
            return_service.handle(&req("alice-returner", "repo/main", 1, return_op("alice-session", generation)), 0)
        });

        let renew_result = renew_handle.join().unwrap();
        let return_result = return_handle.join().unwrap();

        // Whichever order the two calls actually serialized in, both must
        // be individually well-formed, and consistent with one of exactly
        // two legitimate orderings: renew-then-return (renew succeeds,
        // then return succeeds against the renewed lease) or
        // return-then-renew (return succeeds, then renew sees the
        // resource already released - NotHeld, per
        // `renew_after_the_resource_has_been_returned_is_not_held`'s own
        // deterministic pin of that exact case).
        assert!(
            matches!(renew_result, Response::Granted { .. } | Response::Denied(DenyReason::NotHeld)),
            "trial {trial}: renew raced against return produced an unexpected response: {renew_result:?}"
        );
        assert_eq!(return_result, Response::Returned, "trial {trial}: return must always succeed regardless of a concurrent renew attempt");

        let status = service.status("repo/main", 0);
        assert!(status.held.is_none(), "trial {trial}: after a return commits, the resource must be reported free regardless of a concurrent renew's outcome");
    }
}

#[test]
fn a_genuinely_concurrent_identical_retry_is_deduplicated_not_double_executed() {
    const RACERS: usize = 8;

    for trial in 0..CONCURRENCY_TRIALS {
        let service = Arc::new(CheckoutService::new());
        let barrier = Arc::new(Barrier::new(RACERS));

        // Every thread sends the exact same request - client_id, resource,
        // sequence, and op all identical - the precise scenario dedup
        // exists for (a client retrying a request it believes was lost,
        // where the original may or may not have actually landed yet).
        let handles: Vec<_> = (0..RACERS)
            .map(|_| {
                let service = Arc::clone(&service);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    service.handle(&req("alice", "repo/main", 1, checkout_op("alice-session", 60_000)), 0)
                })
            })
            .collect();
        let responses: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Every single response must be identical - dedup that only
        // partially serializes the check/execute/cache sequence can let
        // one racer see the resource already held by a sibling call and
        // return a *different*, incorrect Denied(Held) instead of
        // the one true cached outcome.
        let first = &responses[0];
        for (i, response) in responses.iter().enumerate() {
            assert_eq!(response, first, "trial {trial}: racer {i} got a different response than racer 0 for byte-identical concurrent requests: {response:?} vs {first:?}");
        }
        assert!(matches!(first, Response::Granted { .. }), "trial {trial}: the one true outcome for this request must be Granted, got {first:?}");

        // And the service's own state must agree with that one outcome -
        // alice must genuinely hold the resource, not have "won" a
        // response that was never actually backed by real state.
        let status = service.status("repo/main", 0);
        let held = status.held.unwrap_or_else(|| panic!("trial {trial}: alice's checkout must have actually been recorded, not just returned"));
        assert_eq!(held.holder, "alice-session");
    }
}
