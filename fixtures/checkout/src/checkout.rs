//! Module 02: A Single-Node Checkout Service.
//!
//! The real `Checkout`/`Renew`/`Return`/`Status` API - exclusivity, lease
//! expiry, generation fencing, request deduplication - against a single
//! in-memory node. No replication, no RPC, no network at all: everything
//! that touches this module's own state lives entirely in-process. A
//! deliberately new, richer type set rather than an extension of Module
//! 01's `rpc::CheckoutRequest`/`CheckoutResponse` in place - those already
//! shipped as graded exercise content, and the properties below (generation
//! fencing, per-resource dedup, a capped lease duration) don't fit that
//! narrower shape anyway.
//!
//! `CheckoutService::handle`/`status`/`canonicalize` are stubbed below
//! (`todo!`). Implement them against the provided test suite
//! (`tests/module_02_checkout_service.rs`) - that suite is the
//! deterministic gate, not a spec to read and reimplement from prose.

use std::collections::HashMap;
use std::sync::Mutex;

/// The maximum lease duration a client may request, at `Checkout` or
/// `Renew` time alike - capping only the initial grant and leaving renewal
/// uncapped would reopen the same unbounded-hold abuse one call later.
pub const MAX_LEASE_DURATION_MS: u64 = 5 * 60 * 1000;

/// Reads the current time. Production wires this to a real wall clock;
/// tests wire it to a manually-advanced clock - never read a wall clock
/// directly outside an implementation of this trait, so time stays
/// swappable and deterministic under test (and, once this service runs on
/// top of Raft in a later module, swappable for a replicated logical
/// clock).
pub trait Clock: Send + Sync {
    fn now_ms(&self) -> u64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckoutOp {
    Checkout {
        holder: String,
        lease_duration_ms: u64,
    },
    Renew {
        holder: String,
        generation: u64,
        lease_duration_ms: u64,
    },
    Return {
        holder: String,
        generation: u64,
    },
}

/// Deduplication is keyed by `(client_id, resource, sequence)`, with
/// `sequence` monotonic per `(client_id, resource)` - not a bare opaque ID,
/// and not scoped by `client_id` alone (a client holding two concurrent
/// leases on different resources, and renewing both, needs two independent
/// sequence counters, or one renewal's cache entry would silently clobber
/// the other's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub client_id: String,
    pub resource: String,
    pub sequence: u64,
    pub op: CheckoutOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Granted { generation: u64, expires_at_ms: u64 },
    Returned,
    Denied(DenyReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenyReason {
    /// The resource is currently held, under an unexpired lease, by someone else.
    HeldByOther,
    /// The caller's generation doesn't match the resource's current lease -
    /// a fencing rejection: a stale command against an already-superseded
    /// lease must never be honored.
    StaleGeneration,
    /// `Renew`/`Return` against a resource with no active lease matching the
    /// caller's holder - either it already expired or was already returned.
    NotHeld,
    /// The request's `sequence` is strictly lower than the highest already
    /// recorded for this `(client_id, resource)` pair - a stale, superseded
    /// retry, not a legitimate new request.
    StaleSequence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusInfo {
    pub resource: String,
    pub held: Option<HeldInfo>,
    /// Never reset by a `Return` or a natural expiry - it only ever
    /// increases, so a generation number is never reissued for a given
    /// resource.
    pub last_issued_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldInfo {
    pub holder: String,
    pub generation: u64,
    pub expires_at_ms: u64,
}

pub struct CheckoutService<C: Clock> {
    clock: C,
    #[allow(dead_code)]
    resources: Mutex<HashMap<String, ()>>,
    #[allow(dead_code)]
    dedup: Mutex<HashMap<(String, String), ()>>,
}

impl<C: Clock> CheckoutService<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            resources: Mutex::new(HashMap::new()),
            dedup: Mutex::new(HashMap::new()),
        }
    }

    /// Exposes the injected clock's current reading - never read a wall
    /// clock directly anywhere else in this file.
    pub fn now_ms(&self) -> u64 {
        self.clock.now_ms()
    }

    /// Canonicalizes a resource identifier once, at this API boundary -
    /// every exclusivity/dedup/generation lookup downstream must use this
    /// same canonical value, never a re-parsed one. Without this, two
    /// callers naming the same resource two different ways could bypass
    /// the lock entirely by appearing to hold two different resources.
    ///
    /// Required behavior (see the test suite for the exact checks): at
    /// minimum, leading/trailing whitespace must not create two identities
    /// for the same resource.
    pub fn canonicalize(resource: &str) -> String {
        todo!("implement per this file's doc comments and tests/module_02_checkout_service.rs")
    }

    /// Handles one `Checkout`/`Renew`/`Return` request - the single entry
    /// point every mutating operation goes through (`Status` is a separate,
    /// pure read - see `status` below). Dispatching all three through one
    /// call, rather than three separate methods, is deliberate: a later
    /// module replicates this exact call shape as a single command any
    /// consensus protocol can propose and apply uniformly.
    ///
    /// Required behavior (see the test suite for the exact checks):
    /// - Dedup is checked first, before any other logic: a request whose
    ///   `sequence` exactly matches the highest already recorded for this
    ///   `(client_id, resource)` replays the cached outcome verbatim -
    ///   granted, denied, or returned - without re-executing anything. A
    ///   `sequence` strictly lower than the highest already seen is
    ///   rejected as `DenyReason::StaleSequence`, not re-executed as if new.
    /// - `Checkout` grants exclusive access if the resource has no active,
    ///   unexpired lease; otherwise denies `HeldByOther`. A granted lease
    ///   gets a strictly higher generation than any previously issued for
    ///   this resource - even one whose lease has long since expired or
    ///   been returned; the generation counter is state that outlives any
    ///   single lease.
    /// - `Renew`/`Return` must supply the exact generation the matching
    ///   `Checkout` returned; a mismatch (including a same-generation call
    ///   from the wrong holder) is `DenyReason::StaleGeneration`. A
    ///   generation that matches but whose lease has already lapsed is
    ///   `DenyReason::NotHeld` - a dead lease can't be renewed back to life.
    /// - Both `Checkout` and `Renew` cap `lease_duration_ms` at
    ///   `MAX_LEASE_DURATION_MS` - a client-requested duration beyond that
    ///   is silently capped, not rejected.
    /// - No background timer ever runs. Expiry is derived purely from
    ///   comparing a stored `expires_at_ms` against `self.now_ms()` at the
    ///   moment of each call - an idle resource nobody has touched in a
    ///   while must still be correctly reported as expired the next time
    ///   anything asks, `status` included.
    pub fn handle(&self, request: &Request) -> Response {
        todo!("implement per this file's doc comments and tests/module_02_checkout_service.rs")
    }

    /// A pure read: reports whether `resource` is *currently* held (its
    /// lease hasn't lapsed as of `self.now_ms()` right now), without
    /// mutating or evicting any state - expiry is otherwise only checked
    /// lazily, at the next `Checkout` attempt against that resource, so
    /// this call must independently re-derive the same expiry check for its
    /// own report to be accurate, not trust whatever was last written.
    pub fn status(&self, resource: &str) -> StatusInfo {
        todo!("implement per this file's doc comments and tests/module_02_checkout_service.rs")
    }
}
