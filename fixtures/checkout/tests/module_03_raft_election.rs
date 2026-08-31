//! Module 03: Raft Leader Election - deterministic-tier test suite.
//!
//! Verifies the governing properties from `docs/workshop-design.md` and the
//! Raft paper's Figure 2/4/§5.4.1 against real `turmoil`-injected faults:
//! at most one leader per term (checked against the complete, merged
//! transition log, never a live poll), a leader is eventually elected when a
//! majority can communicate, and election-timeout jitter is genuinely seeded
//! and re-randomized per attempt (not a constant), so the same seed always
//! reproduces the same outcome and a split vote doesn't repeat forever.
//!
//! `SEEDS` below is a small, published, illustrative seed set for this
//! dry-run/exercise stage - not the real practice/held-out split
//! (`docs/workshop-design.md`'s deterministic-gate section) Coachgremlin's
//! grading pass will eventually run against. Passing across this set is not
//! itself a grading criterion.

use checkout::raft::connector::Connector;
use checkout::raft::node::RaftNode;
use checkout::raft::transport::serve_one_rpc;
use checkout::raft::types::{AppendEntriesArgs, InboundMessage, Role, RoleTransitionEvent};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use turmoil::net::{TcpListener, TcpStream};

#[derive(Clone)]
struct TurmoilConnector;

impl Connector for TurmoilConnector {
    type Stream = TcpStream;

    async fn connect(&self, addr: String) -> std::io::Result<Self::Stream> {
        TcpStream::connect(addr).await
    }
}

/// The address a peer *connects to* - the node's turmoil hostname.
fn node_addr(id: u32) -> String {
    format!("node-{id}:9000")
}

/// The address a node itself *binds to* - turmoil, like real sockets,
/// expects a listener to bind "all interfaces on this host," not its own
/// resolved hostname (Module 01's own test established this same split).
fn bind_addr() -> &'static str {
    "0.0.0.0:9000"
}

fn node_host_name(id: u32) -> String {
    format!("node-{id}")
}

/// One entry per observed `AppendEntries` on the wire - the independent,
/// harness-side cross-check that doesn't rely on the learner's own
/// `RoleState` calls being honest. Recorded only when a message is actually
/// *delivered and served*, not merely sent - a partitioned link drops it
/// silently, so this collector goes quiet for an isolated node exactly when
/// the wire can no longer see it, same as any real observer would.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WireLeaderClaim {
    term: u64,
    leader_id: u32,
}

/// Everything a test needs after `spawn_cluster` wires up a cluster: the
/// merged transition-log receiver, the wire-observed-leadership collector,
/// a live pointer to whichever node most recently claimed leadership on the
/// wire, and any background-task failures the harness itself detected.
struct ClusterHandles {
    transition_rx: mpsc::UnboundedReceiver<RoleTransitionEvent>,
    wire_claims: Arc<Mutex<HashSet<WireLeaderClaim>>>,
    current_leader: Arc<Mutex<Option<u32>>>,
    task_failures: Arc<Mutex<Vec<String>>>,
}

impl ClusterHandles {
    fn assert_no_background_task_failures(&self) {
        let failures = self.task_failures.lock().unwrap();
        assert!(
            failures.is_empty(),
            "a node's background task (spawned from RaftNode::start) ended or panicked \
             instead of running for the life of the node: {failures:?}"
        );
    }
}

/// Wires up a `node_count`-node cluster inside `sim`: each node's inbound
/// listener is a `turmoil` host running `RaftNode::start` plus an accept
/// loop.
///
/// Registers a "driver" client that sleeps for `run_for` minus a small
/// margin - `turmoil::Sim::run()` never polls a host's future at all unless
/// at least one client exists (hosts are reactive to clients, not
/// autonomously scheduled), and `simulation_duration` must itself be set
/// somewhat longer than the driver's own sleep, or `Sim::step`'s
/// `elapsed > duration && !is_finished` check can spuriously fire against a
/// perfectly healthy run depending on exactly which tick the driver's sleep
/// resolves on.
fn spawn_cluster(
    sim: &mut turmoil::Sim<'_>,
    node_count: u32,
    sim_seed: u64,
    run_for: Duration,
) -> ClusterHandles {
    let driver_sleep = run_for.saturating_sub(Duration::from_millis(500));
    sim.client("driver", async move {
        tokio::time::sleep(driver_sleep).await;
        Ok(())
    });

    let seq_counter = Arc::new(AtomicU64::new(0));
    let (transition_tx, transition_rx) = mpsc::unbounded_channel();
    let wire_claims: Arc<Mutex<HashSet<WireLeaderClaim>>> = Arc::new(Mutex::new(HashSet::new()));
    // Updated live (not just drained at the end) by the same wire-observation
    // hook, so a test can dynamically discover and isolate whichever node is
    // *actually* leading right now - a fixed node id has no reason to be the
    // one that ends up leading on any given seed.
    let current_leader: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    let task_failures: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let all_ids: Vec<u32> = (0..node_count).collect();
    for &id in &all_ids {
        let peers: BTreeMap<u32, String> = all_ids
            .iter()
            .filter(|&&p| p != id)
            .map(|&p| (p, node_addr(p)))
            .collect();
        let seq_counter = Arc::clone(&seq_counter);
        let transition_tx = transition_tx.clone();
        let wire_claims = Arc::clone(&wire_claims);
        let current_leader = Arc::clone(&current_leader);
        let task_failures = Arc::clone(&task_failures);

        sim.host(node_host_name(id), move || {
            let peers = peers.clone();
            let seq_counter = Arc::clone(&seq_counter);
            let transition_tx = transition_tx.clone();
            let wire_claims = Arc::clone(&wire_claims);
            let current_leader = Arc::clone(&current_leader);
            let task_failures = Arc::clone(&task_failures);
            async move {
                use checkout::raft::types::TransitionLog;

                let transition_log = TransitionLog::new(id, seq_counter, transition_tx);
                let node = Arc::new(RaftNode::new(
                    id,
                    peers,
                    TurmoilConnector,
                    transition_log,
                    sim_seed,
                ));
                let handles = node.start();
                // A correct node's background task(s) run for the node's
                // entire lifetime - if one ever resolves (returns, or
                // panics), that's a real failure the harness must surface,
                // not silently absorb into a misleading "no leader elected."
                for handle in handles {
                    let task_failures = Arc::clone(&task_failures);
                    tokio::spawn(async move {
                        match handle.await {
                            Ok(()) => task_failures.lock().unwrap().push(format!(
                                "node {id}'s background task returned instead of running \
                                 for the node's entire lifetime"
                            )),
                            Err(join_error) => task_failures
                                .lock()
                                .unwrap()
                                .push(format!("node {id}'s background task panicked: {join_error}")),
                        }
                    });
                }

                let listener = TcpListener::bind(bind_addr()).await?;
                loop {
                    let (stream, _) = listener.accept().await?;
                    let node = Arc::clone(&node);
                    let wire_claims = Arc::clone(&wire_claims);
                    let current_leader = Arc::clone(&current_leader);
                    tokio::spawn(async move {
                        let _ = serve_one_rpc(stream, |msg| {
                            let node = Arc::clone(&node);
                            let wire_claims = Arc::clone(&wire_claims);
                            let current_leader = Arc::clone(&current_leader);
                            async move {
                                match msg {
                                    InboundMessage::RequestVote(args) => {
                                        checkout::raft::types::InboundReply::RequestVote(
                                            node.handle_request_vote(args).await,
                                        )
                                    }
                                    InboundMessage::AppendEntries(args) => {
                                        record_wire_claim(&wire_claims, &args);
                                        *current_leader.lock().unwrap() = Some(args.leader_id);
                                        checkout::raft::types::InboundReply::AppendEntries(
                                            node.handle_append_entries(args).await,
                                        )
                                    }
                                }
                            }
                        })
                        .await;
                    });
                }
            }
        });
    }
    ClusterHandles {
        transition_rx,
        wire_claims,
        current_leader,
        task_failures,
    }
}

fn record_wire_claim(wire_claims: &Arc<Mutex<HashSet<WireLeaderClaim>>>, args: &AppendEntriesArgs) {
    wire_claims.lock().unwrap().insert(WireLeaderClaim {
        term: args.term,
        leader_id: args.leader_id,
    });
}

/// Drains every currently-available transition event without blocking - used
/// after `sim.run()` completes (successfully or not - a failed run's partial
/// history is the most useful diagnostic available), when no more events
/// will arrive.
fn drain(rx: &mut mpsc::UnboundedReceiver<RoleTransitionEvent>) -> Vec<RoleTransitionEvent> {
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }
    events.sort_by_key(|e| e.seq);
    events
}

/// Runs `sim` to completion, draining the transition log regardless of
/// outcome (a failed run's partial history is the real diagnostic; discarding
/// it exactly when something went wrong is backwards for a teaching
/// exercise), and panicking with both the real error and that history if the
/// simulation itself failed - "no host/client future panicked" is not the
/// only, or even the most common, way `sim.run()` can return `Err` (an
/// unmet `turmoil::partition`/liveness condition just times out the whole
/// simulation with no panic anywhere).
fn run_and_drain(
    mut sim: turmoil::Sim<'_>,
    rx: &mut mpsc::UnboundedReceiver<RoleTransitionEvent>,
) -> Vec<RoleTransitionEvent> {
    let result = sim.run();
    let events = drain(rx);
    drop(sim); // explicit: ensure the runtime (and every task it owns) is
               // torn down before `rx`'s own drop, not left to whichever
               // order the two locals happen to be declared in.
    if let Err(e) = result {
        panic!(
            "sim.run() returned an error (most commonly: a liveness condition was never met \
             within simulation_duration, not necessarily a panic) - {e}\n\
             {} transition events recorded before failure: {events:?}",
            events.len()
        );
    }
    events
}

/// Checks "at most one leader per term" against the complete history: for
/// each term, the set of nodes that *ever* recorded a Leader transition for
/// that term must never exceed one. Deliberately an absolute check, not an
/// interval-overlap one - Raft's actual guarantee is that a term has at most
/// one leader for all time, not merely that two leaders' tenures don't
/// overlap in the merged sequence. An earlier version of this check cleared
/// a node from a term's "current leaders" set the moment it transitioned
/// away, which made two *sequentially disjoint* leaders for the same term
/// (a real safety violation - e.g. exactly the failure mode a missing
/// one-vote-per-term restriction produces) invisible whenever the first
/// leader stepped down before the second was recorded.
fn assert_at_most_one_leader_per_term(events: &[RoleTransitionEvent]) {
    let mut leaders_by_term: HashMap<u64, HashSet<u32>> = HashMap::new();
    for event in events {
        if event.role == Role::Leader {
            let leaders = leaders_by_term.entry(event.term).or_default();
            leaders.insert(event.node);
            assert!(
                leaders.len() <= 1,
                "term {} had {} different nodes record a Leader transition at some point \
                 (not necessarily overlapping): {:?} (violation surfaced at seq {})",
                event.term,
                leaders.len(),
                leaders,
                event.seq
            );
        }
    }
}

fn assert_a_leader_was_elected(events: &[RoleTransitionEvent]) {
    assert!(
        events.iter().any(|e| e.role == Role::Leader),
        "no node ever became leader: {events:?}"
    );
}

/// Independent, wire-level cross-check: every `(term, leader_id)` pair ever
/// claimed on the wire must have a corresponding "this node became Leader in
/// this term" transition event - this doesn't rely on the learner's own
/// `RoleState` calls being complete or honest, only on the recorded
/// transition log agreeing with what was actually sent over the network.
/// Two different `leader_id`s claiming the same `term` on the wire is a
/// leader-uniqueness violation the merged transition log might miss (e.g. a
/// leader that never got as far as calling `RoleState::set` before crashing)
/// - checked here from the wire evidence alone. Not a substitute for
/// `assert_at_most_one_leader_per_term` (a leader that's isolated before
/// sending a single heartbeat leaves no wire evidence at all) - a redundant,
/// independent signal, not the primary one.
fn assert_wire_claims_agree_with_transition_log(
    events: &[RoleTransitionEvent],
    wire_claims: &HashSet<WireLeaderClaim>,
) {
    let mut claims_by_term: HashMap<u64, HashSet<u32>> = HashMap::new();
    for claim in wire_claims {
        claims_by_term.entry(claim.term).or_default().insert(claim.leader_id);
    }
    for (term, leaders) in &claims_by_term {
        assert!(
            leaders.len() <= 1,
            "term {term} saw AppendEntries on the wire from {} different leader_ids: {leaders:?}",
            leaders.len()
        );
        let claimed_leader = *leaders.iter().next().unwrap();
        assert!(
            events
                .iter()
                .any(|e| e.role == Role::Leader && e.term == *term && e.node == claimed_leader),
            "node {claimed_leader} sent AppendEntries claiming term {term} leadership, \
             but never recorded a Leader transition for that term"
        );
    }
}

const SEEDS: &[u64] = &[1, 2, 3, 5, 8];
/// A larger, dedicated seed set for scenarios specifically trying to force a
/// split vote (see `a_split_vote_still_elects_a_leader_eventually`) - five
/// seeds isn't enough to reliably hit the low-probability case of two
/// candidates timing out within the same narrow window.
const SPLIT_VOTE_SEEDS: std::ops::Range<u64> = 100..130;

#[test]
fn a_lone_node_becomes_its_own_leader() {
    for &seed in SEEDS {
        let run_for = Duration::from_secs(10);
        let sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let mut handles = spawn_cluster_owned(sim, 1, seed, run_for);
        let events = run_and_drain(handles.sim.take().unwrap(), &mut handles.cluster.transition_rx);

        assert_a_leader_was_elected(&events);
        assert_at_most_one_leader_per_term(&events);
        assert_wire_claims_agree_with_transition_log(&events, &handles.cluster.wire_claims.lock().unwrap());
        handles.cluster.assert_no_background_task_failures();
    }
}

#[test]
fn a_three_node_cluster_elects_exactly_one_leader_per_term() {
    for &seed in SEEDS {
        let run_for = Duration::from_secs(20);
        let sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let mut handles = spawn_cluster_owned(sim, 3, seed, run_for);
        let events = run_and_drain(handles.sim.take().unwrap(), &mut handles.cluster.transition_rx);

        assert_a_leader_was_elected(&events);
        assert_at_most_one_leader_per_term(&events);
        let wire_claims = handles.cluster.wire_claims.lock().unwrap();
        assert!(!wire_claims.is_empty(), "no AppendEntries was ever observed on the wire - a leader that never heartbeats is still a bug");
        assert_wire_claims_agree_with_transition_log(&events, &wire_claims);
        drop(wire_claims);
        handles.cluster.assert_no_background_task_failures();
    }
}

/// Latency meaningfully close to the election-timeout floor (unlike
/// `turmoil`'s own ambient default of 0-100ms, already present in every
/// other test in this file) - this is what actually distinguishes this test
/// from `a_three_node_cluster_elects_exactly_one_leader_per_term` rather than
/// nearly duplicating it.
#[test]
fn election_succeeds_under_injected_latency() {
    for &seed in SEEDS {
        let run_for = Duration::from_secs(30);
        let sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .min_message_latency(Duration::from_millis(50))
            .max_message_latency(Duration::from_millis(600))
            .build();
        let mut handles = spawn_cluster_owned(sim, 3, seed, run_for);
        let events = run_and_drain(handles.sim.take().unwrap(), &mut handles.cluster.transition_rx);

        assert_a_leader_was_elected(&events);
        assert_at_most_one_leader_per_term(&events);
        let wire_claims = handles.cluster.wire_claims.lock().unwrap();
        assert!(!wire_claims.is_empty(), "no AppendEntries was ever observed on the wire - a leader that never heartbeats is still a bug");
        assert_wire_claims_agree_with_transition_log(&events, &wire_claims);
        drop(wire_claims);
        handles.cluster.assert_no_background_task_failures();
    }
}

/// The liveness property under a real fault: isolate whichever node is
/// currently leading, and a majority of the *other* nodes must still elect a
/// *new* leader, in a *higher* term, within a bounded window - not merely
/// "some leader exists somewhere in the whole history," which the
/// pre-partition leader alone would already satisfy regardless of what
/// happens afterward (an earlier version of this test asserted exactly
/// that, making its own liveness check unable to fail no matter what the
/// implementation did post-partition). (This does not exercise the
/// held-lock-across-an-outbound-call anti-pattern `node.rs`'s own doc
/// comment warns about via a *fresh* `connect(..)` - this crate's pinned
/// `turmoil` version fails that immediately rather than hanging. It can
/// still be exercised via an *already-established* connection: the isolated
/// leader's own in-flight heartbeat replies are silently dropped by the same
/// partition, and neither `call_append_entries` nor `serve_one_rpc` has a
/// built-in timeout, so a read left waiting on a partitioned peer's reply
/// hangs for real.)
#[test]
fn a_partitioned_minority_leader_never_shares_a_term_with_the_majoritys_new_leader() {
    const NODE_COUNT: u32 = 5;
    for &seed in SEEDS {
        let run_for = Duration::from_secs(60);
        let sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let mut handles = spawn_cluster_owned(sim, NODE_COUNT, seed, run_for);
        let isolated_term: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(None));

        // Isolate whichever node is *actually* leading, discovered live via
        // `current_leader` - not a fixed node id, which has no reason to be
        // the one that ends up leading on any given seed (a fixed choice
        // here previously made this test vacuous: it never forced the
        // majority to hold a fresh election unless the fixed node happened
        // to already be leader). `turmoil::partition` reads a scoped
        // thread-local that's only set while the simulation is actively
        // polling a host/client future - it must be called from inside one,
        // never from the bare test-thread scope between manual `sim.step()`
        // calls.
        let current_leader = Arc::clone(&handles.cluster.current_leader);
        let isolated_term_for_client = Arc::clone(&isolated_term);
        handles.sim.as_mut().unwrap().client("partitioner", async move {
            let leader = loop {
                if let Some(leader) = *current_leader.lock().unwrap() {
                    break leader;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            };
            for other in 0..NODE_COUNT {
                if other != leader {
                    turmoil::partition(node_host_name(leader), node_host_name(other));
                }
            }
            *isolated_term_for_client.lock().unwrap() = Some(leader as u64);
            Ok(())
        });

        let events = run_and_drain(handles.sim.take().unwrap(), &mut handles.cluster.transition_rx);

        assert_a_leader_was_elected(&events);
        assert_at_most_one_leader_per_term(&events);
        assert_wire_claims_agree_with_transition_log(&events, &handles.cluster.wire_claims.lock().unwrap());
        handles.cluster.assert_no_background_task_failures();

        // The actual liveness assertion this test exists for: find the
        // pre-partition leader's own last known term (recorded by the
        // partitioner client itself, at the moment it isolated that node -
        // reusing `isolated_term` as a `NodeId`-shaped stand-in since the
        // partitioner only knows the node id, not its term; look up that
        // node's own highest recorded term from the merged history instead).
        let isolated_node = isolated_term.lock().unwrap().expect("partitioner never found a leader to isolate") as u32;
        let isolated_leader_term = events
            .iter()
            .filter(|e| e.node == isolated_node && e.role == Role::Leader)
            .map(|e| e.term)
            .max()
            .expect("the isolated node was never recorded as leader before being isolated");
        assert!(
            events.iter().any(|e| {
                e.role == Role::Leader && e.node != isolated_node && e.term > isolated_leader_term
            }),
            "no node other than the isolated former leader (node {isolated_node}, last led \
             term {isolated_leader_term}) ever became leader in a higher term - the majority \
             partition never actually held a fresh election: {events:?}"
        );
    }
}

/// A dedicated, larger-seed-set stress test for the specific liveness
/// property Raft's own re-randomization requirement exists to guarantee: a
/// split vote (no candidate reaches a majority in a given term) must not
/// repeat forever. Five seeds (`SEEDS`) aren't enough to reliably hit the
/// narrow-window case of multiple candidates timing out close together;
/// `SPLIT_VOTE_SEEDS` is deliberately larger. This is the closest this
/// suite gets to directly testing "election-timeout jitter is re-randomized
/// per attempt, not a constant" (contract requirement in this module's own
/// `timer.rs`) - it cannot prove re-randomization occurred, only that
/// *if* it didn't, a genuine repeated-split-vote deadlock would eventually
/// show up as a bounded-window liveness failure across enough seeds. A
/// four-node cluster is used deliberately (an even split is topologically
/// possible, unlike an odd-sized cluster).
#[test]
fn a_split_vote_still_elects_a_leader_eventually() {
    for seed in SPLIT_VOTE_SEEDS {
        let run_for = Duration::from_secs(20);
        let sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let mut handles = spawn_cluster_owned(sim, 4, seed, run_for);
        let events = run_and_drain(handles.sim.take().unwrap(), &mut handles.cluster.transition_rx);

        assert_a_leader_was_elected(&events);
        assert_at_most_one_leader_per_term(&events);
        handles.cluster.assert_no_background_task_failures();
    }
}

/// Determinism: the same seed must reproduce a byte-identical transition
/// history every time, across every seed in `SEEDS` - not just one hardcoded
/// value. This is what actually enforces "seed your jitter from
/// `timer::rng_for_node`, kept as your own owned, mutated-in-place state,
/// never re-derived fresh per call" - an implementation that violates this
/// (or uses `HashMap`-ordered iteration, or an un-biased `tokio::select!`)
/// will fail this test, not necessarily any other one. Note this cannot
/// fully enforce re-randomization *within* one run - see
/// `a_split_vote_still_elects_a_leader_eventually` for that half.
#[test]
fn the_same_seed_produces_a_byte_identical_transition_history_twice() {
    let run_once = |seed: u64, run_for: Duration| {
        let sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let mut handles = spawn_cluster_owned(sim, 3, seed, run_for);
        let events = run_and_drain(handles.sim.take().unwrap(), &mut handles.cluster.transition_rx);
        let wire_claims = handles.cluster.wire_claims.lock().unwrap().clone();
        (events, wire_claims)
    };

    let mut all_histories: Vec<Vec<RoleTransitionEvent>> = Vec::new();
    for &seed in SEEDS {
        let run_for = Duration::from_secs(20);
        let (first, first_wire) = run_once(seed, run_for);
        let (second, second_wire) = run_once(seed, run_for);

        assert_a_leader_was_elected(&first);
        assert_at_most_one_leader_per_term(&first);
        assert_wire_claims_agree_with_transition_log(&first, &first_wire);
        assert_eq!(
            first, second,
            "seed {seed} produced two different transition histories - check for unseeded \
             randomness (an rng not stored as your own mutated-in-place field), an un-biased \
             tokio::select!, or HashMap-ordered iteration somewhere in the election logic"
        );
        assert_eq!(
            first_wire, second_wire,
            "seed {seed} produced two different sets of wire-observed AppendEntries claims"
        );
        all_histories.push(first);
    }
    assert!(
        all_histories.windows(2).any(|w| w[0] != w[1]),
        "every seed in SEEDS produced the identical transition history - that's at least as \
         suspicious as two runs of the same seed differing, since it suggests the seed isn't \
         actually influencing the election outcome at all"
    );
}

// --- Harness plumbing to keep `turmoil::Sim` alive alongside its handles ---
//
// `spawn_cluster` takes `&mut Sim` and returns handles that borrow nothing
// from it, but several tests above need to keep mutating `sim` (registering
// the "partitioner" client) after `spawn_cluster` returns, then eventually
// consume it via `run_and_drain`. This tiny wrapper just bundles a
// `Sim` and its `ClusterHandles` together so tests can do both without
// fighting the borrow checker over `sim` being a `&mut` parameter.

struct OwnedCluster<'a> {
    sim: Option<turmoil::Sim<'a>>,
    cluster: ClusterHandles,
}

fn spawn_cluster_owned(
    mut sim: turmoil::Sim<'_>,
    node_count: u32,
    sim_seed: u64,
    run_for: Duration,
) -> OwnedCluster<'_> {
    let cluster = spawn_cluster(&mut sim, node_count, sim_seed, run_for);
    OwnedCluster {
        sim: Some(sim),
        cluster,
    }
}
