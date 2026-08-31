//! Core Raft types (Modules 03-06): domain-agnostic - `LogEntry::command` is
//! opaque bytes, never a `Checkout` type, since the same log replicates any
//! application built on top later (Module 07+). Everything in this file is
//! provided infrastructure, not part of the exercise - Module 03's actual
//! exercise is in `node.rs`. Same crate as Module 01's `rpc.rs`
//! (`fixtures/checkout`), so `crate::rpc::{read_framed, write_framed}`'s
//! `pub(crate)` visibility already covers this module.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub type NodeId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
    // Module 05 will need a way to represent "this node is currently crashed"
    // once turmoil-driven crash simulation exists, so an open leadership
    // interval doesn't appear to stay open forever across a simulated crash.
    // That observation must be synthesized by the TEST HARNESS at the moment
    // it calls `sim.crash()` - a crashed node's own runtime (and everything
    // living in it, including its `RoleState`/`TransitionLog`) is torn down
    // before the node itself could ever report anything about its own crash.
    // Not solved here - Module 03 has no crash simulation at all yet, so this
    // is a deliberate, named deferral for Module 05's own authoring pass, not
    // a gap in this module's own scope.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogEntry {
    pub term: u64,
    pub command: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestVoteArgs {
    pub term: u64,
    pub candidate_id: NodeId,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestVoteReply {
    pub term: u64,
    pub vote_granted: bool,
}

/// Module 03 only ever sends this as an empty heartbeat (`entries: vec![]`) -
/// real log replication is Module 04's exercise. The field exists now, unused
/// by this module's own logic, so Module 04 needs no wire-format change (the
/// same "ship the field before it's needed" precedent Module 01's own types
/// already established).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppendEntriesArgs {
    pub term: u64,
    pub leader_id: NodeId,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<LogEntry>,
    pub leader_commit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppendEntriesReply {
    pub term: u64,
    pub success: bool,
}

/// The wire-level inbound message enum. Module 06 adds `InstallSnapshot` here,
/// not added now, deliberately: every existing `match` on this type is
/// exhaustive, so the compiler forces every *harness-side* dispatch site to
/// account for the new variant at that point. `RaftNode` is a concrete type
/// gaining a new inherent method when that happens, not a trait gaining a new
/// required one. This doesn't generalize to every future addition in this
/// file, though: a new `Role` variant, unlike a new `InboundMessage` variant,
/// would require updating any exhaustive `match self.role_state.role() { .. }`
/// a learner already wrote. That's an accepted, sometimes-intentional part of
/// this workshop's cumulative-project model, not something every later
/// addition is obligated to avoid.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InboundMessage {
    RequestVote(RequestVoteArgs),
    AppendEntries(AppendEntriesArgs),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InboundReply {
    RequestVote(RequestVoteReply),
    AppendEntries(AppendEntriesReply),
}

/// `entries[0]` is always absolute index `first_index`. Starts at 1, per the
/// Raft paper's own 1-based indexing. `first_index_prev_term` is the term of
/// the entry that used to sit at `first_index - 1`, before it (and everything
/// before it) was compacted away - 0 until the first real compaction (Module
/// 06), which is exactly what index 0's term always implicitly was anyway.
/// Already proven correct through this project's own private reference
/// implementation (Modules 03-08): a first design draft's
/// `last_index`/`last_term` unconditionally returned 0/0 on an empty log
/// (only ever correct by coincidence before compaction existed), and
/// `term_at` alone (without `prev_term_for`) caused a real follower/leader
/// livelock the first time a compacted follower's `AppendEntries` check ran.
/// Both are already fixed here. `Serialize`/`Deserialize` are derived now,
/// ahead of Module 05 needing to persist this type, so that already-graded
/// file doesn't need a later breaking change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaftLog {
    entries: Vec<LogEntry>,
    first_index: u64,
    first_index_prev_term: u64,
}

impl Default for RaftLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLookup<'a> {
    Entry(&'a LogEntry),
    OutOfRange,
    Compacted,
}

impl RaftLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            first_index: 1,
            first_index_prev_term: 0,
        }
    }

    pub fn last_index(&self) -> u64 {
        if self.entries.is_empty() {
            self.first_index.saturating_sub(1)
        } else {
            self.first_index + self.entries.len() as u64 - 1
        }
    }

    pub fn last_term(&self) -> u64 {
        if self.entries.is_empty() {
            self.first_index_prev_term
        } else {
            match self.get(self.last_index()) {
                LogLookup::Entry(e) => e.term,
                _ => 0,
            }
        }
    }

    /// The highest index already compacted away, or 0 if nothing ever has
    /// been. Module 04's leader logic must check this *before* trusting an
    /// empty `entries_from` result - an empty `Vec` from `entries_from` is
    /// ambiguous between "nothing new to send" and "the requested start index
    /// was compacted away, send InstallSnapshot instead" (Module 06). This
    /// method is how a caller tells those two cases apart; `entries_from`
    /// alone cannot. (Module 06's own authoring pass still needs to resolve a
    /// related ambiguity this method doesn't cover: `get(0)` reports
    /// `OutOfRange`, never `Compacted`, even after a real compaction - not
    /// fixed here, out of Module 03's own scope.)
    pub fn compacted_boundary(&self) -> u64 {
        self.first_index.saturating_sub(1)
    }

    /// The term to use as `prev_log_term` for `prev_log_index` in your own
    /// AppendEntries consistency check. Use this, never `term_at`, for that
    /// check specifically - `term_at(0)` on an empty, never-compacted log
    /// returns `None`, which would reject every Module 03 heartbeat (whose
    /// `prev_log_index`/`prev_log_term` are both 0) even though a correct
    /// Figure 2 implementation must accept them.
    pub fn prev_term_for(&self, prev_log_index: u64) -> Option<u64> {
        if prev_log_index == self.first_index.saturating_sub(1) {
            Some(self.first_index_prev_term)
        } else {
            self.term_at(prev_log_index)
        }
    }

    pub fn get(&self, absolute_index: u64) -> LogLookup<'_> {
        if absolute_index == 0 {
            return LogLookup::OutOfRange;
        }
        if absolute_index < self.first_index {
            return LogLookup::Compacted;
        }
        let offset = (absolute_index - self.first_index) as usize;
        match self.entries.get(offset) {
            Some(entry) => LogLookup::Entry(entry),
            None => LogLookup::OutOfRange,
        }
    }

    pub fn term_at(&self, absolute_index: u64) -> Option<u64> {
        match self.get(absolute_index) {
            LogLookup::Entry(e) => Some(e.term),
            _ => None,
        }
    }

    pub fn append(&mut self, entry: LogEntry) -> u64 {
        self.entries.push(entry);
        self.last_index()
    }

    /// Drops every entry from `absolute_index` onward. A no-op if
    /// `absolute_index` is already past the end or already compacted -
    /// deliberate, matching Figure 2's own idempotency needs (a repeated or
    /// out-of-order truncate request for an index the log no longer has
    /// opinions about should do nothing, not error).
    pub fn truncate_from(&mut self, absolute_index: u64) {
        if absolute_index < self.first_index {
            return;
        }
        let offset = (absolute_index - self.first_index) as usize;
        self.entries.truncate(offset);
    }

    /// Clones every entry from `absolute_index` (inclusive) onward - what a
    /// leader sends a follower whose `next_index` is `absolute_index`. See
    /// `compacted_boundary`'s doc comment for the ambiguity a caller must
    /// resolve before trusting an empty result here.
    pub fn entries_from(&self, absolute_index: u64) -> Vec<LogEntry> {
        if absolute_index < self.first_index {
            return Vec::new();
        }
        let offset = (absolute_index - self.first_index) as usize;
        self.entries.get(offset..).unwrap_or(&[]).to_vec()
    }

    /// Checked compaction (Module 06): trims a prefix this log already holds
    /// consistently. A no-op (returns `false`) unless the log has a real
    /// entry at `up_to_index` whose term matches `up_to_term`. Drops every
    /// entry `<= up_to_index` and can never move `first_index` backward.
    pub fn compact(&mut self, up_to_index: u64, up_to_term: u64) -> bool {
        if up_to_index <= self.first_index.saturating_sub(1) {
            return false;
        }
        match self.term_at(up_to_index) {
            Some(term) if term == up_to_term => {
                let offset = (up_to_index - self.first_index) as usize + 1;
                self.entries.drain(0..offset);
                self.first_index = up_to_index + 1;
                self.first_index_prev_term = up_to_term;
                true
            }
            _ => false,
        }
    }

    /// Forceful reset (Module 06): used only when the log does *not* already
    /// hold `last_included_index` consistently (a real InstallSnapshot from a
    /// conflicting or too-short follower log). A no-op if
    /// `last_included_index` would move the boundary backward or sideways;
    /// otherwise discards every current entry and re-anchors the log at
    /// `last_included_index + 1`. (Module 06's own authoring pass still needs
    /// to guard the `+ 1` against a `last_included_index == u64::MAX` caller
    /// - not fixed here, out of Module 03's own scope.)
    pub fn force_reset_to(&mut self, last_included_index: u64, last_included_term: u64) -> bool {
        if last_included_index <= self.first_index.saturating_sub(1) {
            return false;
        }
        self.entries.clear();
        self.first_index = last_included_index + 1;
        self.first_index_prev_term = last_included_term;
        true
    }
}

/// One entry in a node's append-only, in-process observability log. `seq`
/// comes from a fixture-owned counter shared across every node in one
/// simulation, never something your own code generates - it's how the test
/// harness merges every node's own per-node stream into one total order to
/// check "at most one leader per term" continuously, not just at test end.
/// Received events must be sorted by `seq` before analysis - the counter
/// increment and the channel send are two separate steps, not one atomic
/// operation, so two nodes' events can be delivered out of `seq` order even
/// though this happens to not occur under `turmoil`'s own single-threaded
/// stepping today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleTransitionEvent {
    pub seq: u64,
    pub node: NodeId,
    pub term: u64,
    pub role: Role,
}

/// Provided. Owns the mechanics of recording a role/term change - one logical
/// instance per node, constructed by the test harness and handed to that
/// node's own constructor. `Clone` (every clone of one node's `TransitionLog`
/// shares the same `node` id, the same sequence counter, and the same
/// underlying channel) - not because instances are shared *across* nodes
/// (they aren't; the harness constructs a distinct one per node), but because
/// `turmoil::Sim::host`'s factory closure runs again on every `sim.bounce()`
/// and needs its own captured copy to hand to a freshly (re)constructed node
/// each time. Not constructed by you; [`RoleState`] below is what you
/// actually hold and call.
#[derive(Debug, Clone)]
pub struct TransitionLog {
    node: NodeId,
    seq_counter: Arc<AtomicU64>,
    tx: UnboundedSender<RoleTransitionEvent>,
}

impl TransitionLog {
    pub fn new(
        node: NodeId,
        seq_counter: Arc<AtomicU64>,
        tx: UnboundedSender<RoleTransitionEvent>,
    ) -> Self {
        Self {
            node,
            seq_counter,
            tx,
        }
    }

    fn record(&self, term: u64, role: Role) {
        let seq = self.seq_counter.fetch_add(1, Ordering::SeqCst);
        self.tx
            .send(RoleTransitionEvent {
                seq,
                node: self.node,
                term,
                role,
            })
            .expect(
                "TransitionLog receiver closed mid-simulation - a harness bug, not a condition to swallow silently",
            );
    }
}

/// Provided. Owns your node's `term`/`role` - the only way to read or change
/// either is through this type, so recording a transition isn't a manual step
/// you could forget: [`set`](RoleState::set) always records, in the same
/// call, before returning, whenever the role or term actually changes.
/// Construct one (via [`RoleState::new`]) as part of your own state, seeded
/// with the `TransitionLog` your node's constructor receives - construction
/// itself emits an initial `Follower@0` baseline event, so every node's
/// merged stream has a defined starting point, including immediately after a
/// `bounce()` reconstructs it.
///
/// This does not, by itself, prove you called `set` at the *right* moment
/// (before replying to an RPC, before spawning async work) - that discipline
/// is still yours to hold. It also does not own `voted_for`: advancing to a
/// higher term must reset your own `voted_for` to `None` in the same logical
/// step, and `RoleState` won't do that for you - `voted_for` isn't part of
/// what needs to be recorded for observability, only `term`/`role` are.
#[derive(Debug)]
pub struct RoleState {
    term: u64,
    role: Role,
    log: TransitionLog,
}

impl RoleState {
    pub fn new(log: TransitionLog) -> Self {
        log.record(0, Role::Follower);
        Self {
            term: 0,
            role: Role::Follower,
            log,
        }
    }

    pub fn term(&self) -> u64 {
        self.term
    }

    pub fn role(&self) -> Role {
        self.role
    }

    /// Updates term/role and records the transition, in that order, in this
    /// one call - iff either actually changed (calling `set` with the same
    /// term and role as before is a harmless no-op, not a spurious event).
    pub fn set(&mut self, term: u64, role: Role) {
        if term != self.term || role != self.role {
            self.term = term;
            self.role = role;
            self.log.record(term, role);
        }
    }
}
