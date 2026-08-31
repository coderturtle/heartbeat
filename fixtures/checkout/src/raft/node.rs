//! Module 03: Raft Leader Election.
//!
//! Implement the `todo!()`s on `RaftNode` below. This module's fixed surface
//! (`types`, `timer`, `connector`, `transport`) hides wire framing,
//! connection management, and timing constants - same as Module 01 already
//! proved the framing layer works. What's left, and what this module is
//! actually about: the Figure 2 election algorithm, and how you structure
//! concurrent access to your own node's state to make it correct under real,
//! unreliable timing.
//!
//! **You choose your own interior-mutability/synchronization strategy** - a
//! `Mutex<YourState>`, an `RwLock`, or a channel-fed actor task are all fine.
//! Background work is spawned via `tokio::spawn` (not `spawn_local`), so
//! anything it touches must be `Send + 'static` - an `Rc<RefCell<_>>`-based
//! design will not compile once spawned this way. Nothing in the test suite
//! inspects your internal structure beyond that; it only observes behavior.
//!
//! **A real trap, not scaffolded away:** `transport::call_request_vote`/
//! `call_append_entries`, and `connector.connect(..)` itself, have no
//! built-in timeout, deliberately - wrap a call in `tokio::time::timeout(..)`
//! yourself when you don't want one partitioned or slow peer to block your
//! own progress. The specific failure this module's own test suite checks
//! for: holding your state's lock across an outbound call. `turmoil`'s
//! partition scenarios will deadlock a node built that way the first time a
//! peer is cut off mid-connect-or-mid-RPC, and a majority of the *other*
//! nodes must still elect a leader within a bounded window regardless.
//!
//! **Two-phase construction.** [`new`](RaftNode::new) returns a plain,
//! not-yet-running `Self` and must not spawn anything - nothing can yet hold
//! an `Arc` pointing back at it. The harness calls
//! `Arc::new(RaftNode::new(..))`, then calls [`start`](RaftNode::start) on
//! that `Arc`, and only *after* `start` returns does its own accept loop
//! begin serving inbound RPCs - so a channel-fed actor you spawn in `start`
//! is guaranteed a live consumer before any request can arrive. Construct
//! your actual Figure 2 state (`RoleState`, `voted_for`, `RaftLog`) either as
//! a field behind a `Mutex`/`RwLock` set up in `new` (reachable from both
//! `start` and the RPC handlers via `&self`), or as local variables owned
//! entirely by a task you spawn in `start` (an actor model, using a *clone*
//! of `self.transition_log` to construct your own `RoleState` inside that
//! task). Both `new` and `start` are (re-)invoked from inside the test
//! harness's own `sim.host(..)` factory closure - on every `sim.bounce()`,
//! not just once - so anything spawned outside that closure escapes
//! simulated time and determinism breaks wholesale. `start` returns its
//! spawned tasks' `JoinHandle`s so the harness can detect a panicked
//! background task instead of silently reading "no leader elected."
//!
//! **Determinism, beyond the obvious:** seed your own election-timeout
//! jitter from [`timer::rng_for_node`]/[`timer::next_election_timeout`] -
//! call `rng_for_node` once in `new`, store the result, never re-seed. Use
//! `tokio::time::{sleep, Instant, interval}`, never `std::time`/
//! `std::thread::sleep` - only the former is patched by `turmoil`. Note that
//! `tokio::time::interval`'s default `MissedTickBehavior::Burst` fires an
//! immediate first tick and can emit a catch-up burst after any stall - set
//! `MissedTickBehavior::Delay` if you use it for your heartbeat loop.
//! `peers` below is a `BTreeMap`, not a `HashMap` - never restructure it into
//! an unordered collection for anything you iterate during a run (this
//! project has hit exactly that nondeterminism bug before: identical seed,
//! different RPC dispatch order, different outcome). If you use
//! `tokio::select!` for your own event loop, put `biased;` first - its
//! default branch-polling order draws from a thread-local RNG `turmoil` does
//! not seed. The test suite runs the same seed twice and requires a
//! byte-identical `RoleTransitionEvent` sequence both times.
//!
//! **Your term/role live in [`RoleState`], not a field of your own** - route
//! every read and write through it (`self.role_state.term()`/`.role()`/
//! `.set(term, role)`), so a transition can't slip out unrecorded through a
//! separate, driftable copy of the same information. `RoleState` does not
//! own `voted_for`: advancing to a higher term must reset your own
//! `voted_for` to `None` in the same logical step, and `RoleState` won't do
//! that for you.

use super::connector::Connector;
use super::timer::{rng_for_node, DeterministicRng};
use super::types::{
    AppendEntriesArgs, AppendEntriesReply, NodeId, RequestVoteArgs, RequestVoteReply,
    TransitionLog,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::task::JoinHandle;

// You'll also need, once you use them: `super::types::{RaftLog, Role, RoleState}`
// and `super::timer::next_election_timeout` - not imported above since nothing
// in this unimplemented stub references them yet.

/// Illustrative placeholder for your own internal state - not a prescribed
/// layout, same as Module 02's `ServiceState`. `role_state` (term/role) and
/// `log: RaftLog` (add this field yourself, behind whatever
/// interior-mutability wrapper you choose) are Figure 2's persistent/volatile
/// state.
#[allow(dead_code)]
pub struct RaftNode<C: Connector> {
    id: NodeId,
    peers: BTreeMap<NodeId, String>,
    connector: C,
    transition_log: TransitionLog,
    rng: DeterministicRng,
    // Your own fields: role_state (behind a Mutex/RwLock, or owned entirely
    // by a task you spawn in `start`), voted_for, log: RaftLog, whatever else
    // you need.
}

impl<C: Connector> RaftNode<C> {
    /// Must not spawn anything - see this file's module doc on two-phase
    /// construction. Store `transition_log` and the RNG from `rng_for_node`
    /// as your own fields here; construct `RoleState`/`RaftLog` either here
    /// (behind a lock) or inside `start` (for an actor model).
    pub fn new(
        id: NodeId,
        peers: BTreeMap<NodeId, String>,
        connector: C,
        transition_log: TransitionLog,
        sim_seed: u64,
    ) -> Self {
        let rng = rng_for_node(sim_seed, id);
        Self {
            id,
            peers,
            connector,
            transition_log,
            rng,
        }
    }

    /// Spawn your own background election-timeout task (and anything else
    /// that needs to run continuously) here, using a cloned `Arc<Self>` so it
    /// can call back into your own methods. Return every `JoinHandle` you
    /// create so the harness can retain them.
    pub fn start(self: &Arc<Self>) -> Vec<JoinHandle<()>> {
        let _ = (&self.peers, &self.connector, &self.transition_log, self.rng);
        todo!("implement per this file's doc comments and tests/module_03_raft_election.rs")
    }

    pub async fn handle_request_vote(&self, args: RequestVoteArgs) -> RequestVoteReply {
        let _ = args;
        todo!("implement per this file's doc comments and tests/module_03_raft_election.rs")
    }

    pub async fn handle_append_entries(&self, args: AppendEntriesArgs) -> AppendEntriesReply {
        let _ = args;
        todo!("implement per this file's doc comments and tests/module_03_raft_election.rs")
    }
}
