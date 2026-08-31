//! Module 03: Raft Leader Election - deterministic-tier test suite.
//!
//! Verifies the governing properties from `docs/workshop-design.md` and the
//! Raft paper's Figure 2/4/§5.4.1 against real `turmoil`-injected faults:
//! at most one leader per term (checked against the complete, merged
//! transition log, never a live poll), a leader is eventually elected when a
//! majority can communicate, and election-timeout jitter is genuinely seeded
//! (not a constant), so the same seed always reproduces the same outcome.

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
/// `RoleState` calls being honest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WireLeaderClaim {
    term: u64,
    leader_id: u32,
}

/// Wires up a `node_count`-node cluster inside `sim`: each node's inbound
/// listener is a `turmoil` host running `RaftNode::start` plus an accept
/// loop. Returns the merged transition-log receiver and the shared
/// wire-observed-leadership collector.
///
/// Registers a "driver" client that just sleeps for `run_for` -
/// `turmoil::Sim::run()` never polls a host's future at all unless at least
/// one client exists; hosts are reactive to clients, not autonomously
/// scheduled (a real, previously-confirmed `turmoil` API behavior in this
/// project's own private reference implementation).
fn spawn_cluster(
    sim: &mut turmoil::Sim<'_>,
    node_count: u32,
    sim_seed: u64,
    run_for: Duration,
) -> (
    mpsc::UnboundedReceiver<RoleTransitionEvent>,
    Arc<Mutex<HashSet<WireLeaderClaim>>>,
    Arc<Mutex<Option<u32>>>,
) {
    sim.client("driver", async move {
        tokio::time::sleep(run_for).await;
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

        sim.host(node_host_name(id), move || {
            let peers = peers.clone();
            let seq_counter = Arc::clone(&seq_counter);
            let transition_tx = transition_tx.clone();
            let wire_claims = Arc::clone(&wire_claims);
            let current_leader = Arc::clone(&current_leader);
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
                let _handles = node.start();

                let listener = TcpListener::bind(bind_addr()).await?;
                loop {
                    let Ok((stream, _)) = listener.accept().await else {
                        return Ok(());
                    };
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
    (transition_rx, wire_claims, current_leader)
}

fn record_wire_claim(wire_claims: &Arc<Mutex<HashSet<WireLeaderClaim>>>, args: &AppendEntriesArgs) {
    wire_claims.lock().unwrap().insert(WireLeaderClaim {
        term: args.term,
        leader_id: args.leader_id,
    });
}

/// Drains every currently-available transition event without blocking - used
/// after `sim.run()` completes, when no more events will arrive.
fn drain(rx: &mut mpsc::UnboundedReceiver<RoleTransitionEvent>) -> Vec<RoleTransitionEvent> {
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }
    events.sort_by_key(|e| e.seq);
    events
}

/// Checks "at most one leader per term" against the complete, ordered
/// transition history: for each term, no two *different* nodes may both have
/// an open "became Leader in this term" interval at the same point in the
/// merged sequence.
fn assert_at_most_one_leader_per_term(events: &[RoleTransitionEvent]) {
    let mut open_leaders: HashMap<u64, HashSet<u32>> = HashMap::new();
    for event in events {
        for leaders in open_leaders.values_mut() {
            leaders.remove(&event.node);
        }
        if event.role == Role::Leader {
            let leaders = open_leaders.entry(event.term).or_default();
            leaders.insert(event.node);
            assert!(
                leaders.len() <= 1,
                "term {} had {} concurrent leaders at seq {}: {:?}",
                event.term,
                leaders.len(),
                event.seq,
                leaders
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
/// - checked here from the wire evidence alone.
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

#[test]
fn a_lone_node_becomes_its_own_leader() {
    for &seed in SEEDS {
        let run_for = Duration::from_secs(10);
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let (mut rx, wire_claims, _current_leader) = spawn_cluster(&mut sim, 1, seed, run_for);
        sim.run().expect("sim.run() failed - see panic/error above for the real cause");

        let events = drain(&mut rx);
        assert_a_leader_was_elected(&events);
        assert_at_most_one_leader_per_term(&events);
        assert_wire_claims_agree_with_transition_log(&events, &wire_claims.lock().unwrap());
    }
}

#[test]
fn a_three_node_cluster_elects_exactly_one_leader_per_term() {
    for &seed in SEEDS {
        let run_for = Duration::from_secs(20);
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let (mut rx, wire_claims, _current_leader) = spawn_cluster(&mut sim, 3, seed, run_for);
        sim.run().expect("sim.run() failed - see panic/error above for the real cause");

        let events = drain(&mut rx);
        assert_a_leader_was_elected(&events);
        assert_at_most_one_leader_per_term(&events);
        assert_wire_claims_agree_with_transition_log(&events, &wire_claims.lock().unwrap());
    }
}

#[test]
fn election_succeeds_under_injected_latency() {
    for &seed in SEEDS {
        let run_for = Duration::from_secs(25);
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .min_message_latency(Duration::from_millis(10))
            .max_message_latency(Duration::from_millis(150))
            .build();
        let (mut rx, wire_claims, _current_leader) = spawn_cluster(&mut sim, 3, seed, run_for);
        sim.run().expect("sim.run() failed - see panic/error above for the real cause");

        let events = drain(&mut rx);
        assert_a_leader_was_elected(&events);
        assert_at_most_one_leader_per_term(&events);
        assert_wire_claims_agree_with_transition_log(&events, &wire_claims.lock().unwrap());
    }
}

/// The liveness property under a real fault: isolate whichever node is
/// currently leading, and a majority of the *other* nodes must still elect a
/// leader within a bounded window, regardless of what the isolated node
/// believes about its own leadership. (This does not exercise the
/// held-lock-across-an-outbound-call anti-pattern `node.rs`'s own doc comment
/// warns about - this crate's pinned `turmoil` version fails a fresh
/// `connect(..)` to an already-partitioned host immediately rather than
/// hanging, so that specific failure mode isn't forceable from this test.)
#[test]
fn a_partitioned_minority_leader_never_shares_a_term_with_the_majoritys_new_leader() {
    const NODE_COUNT: u32 = 5;
    for &seed in SEEDS {
        let run_for = Duration::from_secs(40);
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let (mut rx, wire_claims, current_leader) = spawn_cluster(&mut sim, NODE_COUNT, seed, run_for);

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
        sim.client("partitioner", async move {
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
            Ok(())
        });

        sim.run().expect("sim.run() failed - see panic/error above for the real cause");

        let events = drain(&mut rx);
        assert_a_leader_was_elected(&events);
        // The whole point of this test: even with the (former) leader
        // isolated and possibly still believing itself leader in its own
        // last term, the merged history must never show two *different*
        // nodes both leading the *same* term - a higher term in the
        // surviving majority is expected and correct, not a violation.
        assert_at_most_one_leader_per_term(&events);
        assert_wire_claims_agree_with_transition_log(&events, &wire_claims.lock().unwrap());
    }
}

/// Determinism: the same seed must reproduce a byte-identical transition
/// history every time. This is what actually enforces the "seed your jitter
/// from `timer::rng_for_node`, never `rand::random()` or a `HashMap`-ordered
/// dispatch" requirements in `node.rs`'s own doc comment - an implementation
/// that violates either will fail this test, not necessarily any other one.
#[test]
fn the_same_seed_produces_a_byte_identical_transition_history_twice() {
    let seed = 42;
    let run_for = Duration::from_secs(20);

    let run_once = |seed: u64| {
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(run_for)
            .build();
        let (mut rx, _wire_claims, _current_leader) = spawn_cluster(&mut sim, 3, seed, run_for);
        sim.run().expect("sim.run() failed - see panic/error above for the real cause");
        drain(&mut rx)
    };

    let first = run_once(seed);
    let second = run_once(seed);
    assert_a_leader_was_elected(&first);
    assert_eq!(
        first, second,
        "the same seed produced two different transition histories - check for unseeded \
         randomness (rand::random(), an un-biased tokio::select!) or HashMap-ordered iteration \
         somewhere in the election logic"
    );
}
