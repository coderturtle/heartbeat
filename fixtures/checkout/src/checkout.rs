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
//! narrower shape anyway. `Serialize`/`Deserialize` are derived on every
//! public request/response/status type now (`Request`, `CheckoutOp`,
//! `Response`, `DenyReason`, `StatusInfo`, `HeldInfo`), even though nothing
//! in this module sends them over a wire - Module 07 replicates this exact
//! call shape through a Raft log later, `Status` included (the governing
//! design permits routing a read through the log as an alternative to a
//! separate read-index protocol), and a type that can't serialize would
//! force a change to this already-graded file at that point. This module's
//! own snapshot/compaction story (what Module 07's own persistence layer
//! must be able to export and restore to keep a resource's generation
//! counter alive across a restart) is that module's job to build, not
//! this one's - this module's job is making sure the underlying invariant
//! (the generation counter itself genuinely persists, independent of any
//! active lease) is real and tested now, which it is.
//!
//! `CheckoutService::handle`/`status`/`canonicalize` are stubbed below
//! (`todo!`). Implement them against the provided test suite
//! (`tests/module_02_checkout_service.rs`) - that suite is the
//! deterministic gate, not a spec to read and reimplement from prose.
//!
//! `handle`/`status` both take `now_ms` as an explicit parameter, not from
//! an internally-owned clock - deliberate, so this exact function signature
//! stays usable once a caller's notion of "now" isn't its own local clock
//! read at all, but a timestamp a Raft leader already committed into the
//! log entry being applied. A single-node caller (this module's own test
//! suite included) just passes its own clock's current reading at each
//! call site instead. Neither function checks that `now_ms` is
//! non-decreasing across calls - this service trusts its caller for that,
//! the same way it trusts a single-node caller not to call it from two
//! threads with wildly different clocks; a caller with its own real
//! ordering guarantee (a replicated log applying entries in order, in a
//! later module) gets this for free, but nothing here enforces it in
//! isolation.
//!
//! `client_id` is a dedup-scoping key only - it is never checked as an
//! authorization credential. `Renew`/`Return` are authorized purely by
//! `(holder, generation)` matching the resource's current lease; the
//! `client_id` on that same request can be anything, including a different
//! `client_id` than the one that originally called `Checkout`. This is
//! deliberate (a lease's bearer is whoever holds `holder`+`generation`, not
//! whoever happened to dial in the original request), not an oversight -
//! stated explicitly here because it's easy to assume otherwise.

use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// The maximum lease duration a client may request, at `Checkout` or
/// `Renew` time alike - capping only the initial grant and leaving renewal
/// uncapped would reopen the same unbounded-hold abuse one call later.
pub const MAX_LEASE_DURATION_MS: u64 = 5 * 60 * 1000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Deduplication is keyed by `(client_id, resource)`, with `sequence`
/// checked (monotonic per that pair) - not a bare opaque ID, and not
/// scoped by `client_id` alone (a client holding two concurrent leases on
/// different resources, and renewing both, needs two independent sequence
/// counters, or one renewal's cache entry would silently clobber the
/// other's). Only the single highest sequence and its response are ever
/// retained per `(client_id, resource)` - not a growing table keyed by
/// every sequence ever seen, which would never be safe to bound.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub client_id: String,
    pub resource: String,
    pub sequence: u64,
    pub op: CheckoutOp,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Response {
    Granted { generation: u64, expires_at_ms: u64 },
    Returned,
    Denied(DenyReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenyReason {
    /// The resource is currently held, under an unexpired lease, by anyone,
    /// deliberately not named `HeldByOther`: this is also the response to
    /// a caller re-`Checkout`-ing a resource *they themselves* already hold
    /// under a *new* request (a different `sequence`), since a caller who
    /// wants to keep an existing lease alive calls `Renew`, not a fresh
    /// `Checkout`. This service does not special-case "checking out
    /// something you already hold" as anything other than "still held."
    Held,
    /// The caller's `(holder, generation)` pair doesn't authorize acting on
    /// the resource's current lease - a fencing rejection covering two
    /// distinct cases deliberately folded into one reason, not two: the
    /// generation itself doesn't match (a stale command against an
    /// already-superseded lease), or the generation matches but the
    /// supplied `holder` doesn't (an unreachable case if generations are
    /// only ever handed to the caller that requested them, but treated as
    /// a fencing violation rather than assumed impossible). Both describe
    /// the same underlying fact - the caller doesn't hold the authority
    /// over this lease it's claiming to - so neither `Renew` nor `Return`
    /// distinguishes them further.
    StaleGeneration,
    /// `Renew`/`Return` against a resource with no active lease matching the
    /// caller's holder - either it already expired, was already returned,
    /// or (for `Renew` specifically) the supplied generation matches but
    /// the lease it belonged to has since lapsed. Both `Renew` and `Return`
    /// treat an expired-but-matching-generation lease identically to one
    /// that's already been returned - a dead lease can't be renewed *or*
    /// explicitly returned back to life.
    NotHeld,
    /// The request's `sequence` is strictly lower than the highest already
    /// recorded for this `(client_id, resource)` pair - a stale, superseded
    /// retry, not a legitimate new request.
    StaleSequence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusInfo {
    /// The canonical form of the resource identifier that was queried -
    /// never the raw, possibly-unnormalized argument `status` was called
    /// with.
    pub resource: String,
    pub held: Option<HeldInfo>,
    /// Never reset by a `Return` or a natural expiry - it only ever
    /// increases, so a generation number is never reissued for a given
    /// resource. Scoped per resource, not global - a fresh resource's
    /// first grant is always generation 1, independent of how many
    /// generations any other resource has issued.
    pub last_issued_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldInfo {
    pub holder: String,
    pub generation: u64,
    pub expires_at_ms: u64,
}

// The three private structs below are illustrative placeholders, not a
// prescribed layout - reshape them however `handle`/`status` actually need
// (in particular, `ResourceState::active` as written can't hold anything
// resembling a real lease; that's intentional, not a hint about the
// intended shape).
struct ResourceState {
    #[allow(dead_code)]
    last_issued_generation: u64,
    #[allow(dead_code)]
    active: Option<()>,
}

struct DedupEntry {
    #[allow(dead_code)]
    sequence: u64,
    #[allow(dead_code)]
    response: Response,
}

struct ServiceState {
    #[allow(dead_code)]
    resources: HashMap<String, ResourceState>,
    #[allow(dead_code)]
    dedup: HashMap<(String, String), DedupEntry>,
}

/// A single `Mutex` guarding *all* of this service's state, locked once for
/// the full duration of a `handle`/`status` call - not one lock per map.
/// Two maps behind two separate locks, each acquired and released
/// independently around the dedup check, the actual operation, and the
/// dedup insert, is exactly the shape that lets two concurrent, identical
/// retries both slip past the dedup check and both execute for real
/// (found the hard way, via this module's own dry run: a `Checkout`
/// retried concurrently could come back `Granted` on one thread and
/// `Denied(Held)` on the other, with whichever `dedup` write lands last
/// permanently overwriting the cached outcome for the one that actually
/// holds the lease). One lock, held across the whole operation, is what
/// makes "check the cache, do the work, update the cache" atomic with
/// respect to any other concurrent call for the same key.
pub struct CheckoutService {
    #[allow(dead_code)]
    state: Mutex<ServiceState>,
}

impl CheckoutService {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ServiceState {
                resources: HashMap::new(),
                dedup: HashMap::new(),
            }),
        }
    }

    /// Canonicalizes an identifier once, at this API boundary - every
    /// exclusivity/dedup/generation lookup downstream must use this same
    /// canonical value, never a re-parsed one. Without this, two callers
    /// naming the same resource two different ways could bypass the lock
    /// entirely by appearing to hold two different resources. This applies
    /// to *both halves* of the dedup key, `resource` and `client_id` alike:
    /// a retry whose `client_id` or `resource` string happens to be spelled
    /// with different surrounding whitespace than the original call must
    /// still be recognized as the same `(client_id, resource)` pair.
    /// `holder` is deliberately not canonicalized anywhere in this service,
    /// since it's compared as an opaque bearer credential, not a
    /// human-facing name that needs normalizing.
    ///
    /// Required behavior (see the test suite for the exact checks): at
    /// minimum, leading/trailing whitespace must not create two identities
    /// for the same string.
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
    /// `now_ms` is supplied by the caller, not read from any clock this
    /// service owns, or any other ambient source - given the same
    /// `request`, `now_ms`, and prior sequence of committed calls, the
    /// result must always be the same (dedup's own cached-outcome replay
    /// depends on exactly this: the accumulated state this function itself
    /// builds up is a legitimate input to its own next call, a hidden
    /// clock or source of randomness is not).
    ///
    /// Required behavior (see the test suite for the exact checks):
    /// - The dedup check, the operation itself, and recording the new
    ///   dedup entry must happen as one atomic unit with respect to any
    ///   other concurrent call - see `ServiceState`'s own doc comment for
    ///   why two separately-locked critical sections are not equivalent to
    ///   this, even though it might look that way at a glance.
    /// - A request whose `sequence` exactly matches the highest already
    ///   recorded for this `(client_id, resource)` replays the cached
    ///   outcome verbatim - granted, denied, or returned - without
    ///   re-executing anything, even if the world has changed since (a
    ///   lease has expired, a different client now holds the resource,
    ///   the clock has advanced). A `sequence` strictly lower than the
    ///   highest already seen is rejected as `DenyReason::StaleSequence`,
    ///   not re-executed as if new.
    /// - `Checkout` grants exclusive access if the resource has no active,
    ///   unexpired lease (`expires_at_ms > now_ms`, strictly - a lease
    ///   whose `expires_at_ms` exactly equals `now_ms` has lapsed);
    ///   otherwise denies `DenyReason::Held` - including when the caller
    ///   already holds the resource themselves under a still-live lease
    ///   and is attempting a fresh `Checkout` (not a `Renew`) against it. A
    ///   granted lease gets a strictly higher generation than any
    ///   previously issued for this resource - even one whose lease has
    ///   long since expired or been returned; the generation counter is
    ///   state that outlives any single lease, and is scoped per resource.
    /// - `Renew`/`Return` must supply the exact generation the matching
    ///   `Checkout` returned; a mismatch (including a same-generation call
    ///   from the wrong holder) is `DenyReason::StaleGeneration`. A
    ///   generation that matches but whose lease has already lapsed - or
    ///   that has already been explicitly returned - is
    ///   `DenyReason::NotHeld` for both `Renew` and `Return` alike; a dead
    ///   lease can't be renewed or returned back to life.
    /// - Both `Checkout` and `Renew` cap `lease_duration_ms` at
    ///   `MAX_LEASE_DURATION_MS` - a client-requested duration beyond that
    ///   is silently capped, not rejected, and the cap applies to the
    ///   *requested duration*, not to the resulting absolute timestamp (a
    ///   lease granted at `now_ms = 1_000_000` with an oversized requested
    ///   duration still gets a full `MAX_LEASE_DURATION_MS` of life from
    ///   that point, not an already-lapsed `expires_at_ms` clamped down to
    ///   the constant's own raw value).
    /// - No background timer ever runs, and this function must never read
    ///   any clock other than the `now_ms` it was called with. Expiry is
    ///   derived purely from comparing a stored `expires_at_ms` against
    ///   `now_ms` - an idle resource nobody has touched in a while must
    ///   still be correctly reported as expired the next time anything
    ///   asks, `status` included, using the exact same comparison.
    pub fn handle(&self, request: &Request, now_ms: u64) -> Response {
        todo!("implement per this file's doc comments and tests/module_02_checkout_service.rs")
    }

    /// A pure read: reports whether `resource` is *currently* held (its
    /// lease hasn't lapsed as of `now_ms`), without mutating or evicting
    /// any state whatsoever - not the resource's own entry, not its
    /// generation counter, nothing. Expiry is otherwise only checked
    /// lazily, at the next `Checkout` attempt against that resource, so
    /// this call must independently re-derive the exact same expiry check
    /// `handle` uses for its own report to be accurate, not trust whatever
    /// was last written, and must never take a shortcut (such as evicting
    /// an expired resource's entry on read) that `handle`'s own generation-
    /// persistence requirement forbids.
    pub fn status(&self, resource: &str, now_ms: u64) -> StatusInfo {
        todo!("implement per this file's doc comments and tests/module_02_checkout_service.rs")
    }
}

impl Default for CheckoutService {
    fn default() -> Self {
        Self::new()
    }
}
