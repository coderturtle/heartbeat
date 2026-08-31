//! Provided election-timeout jitter and timing constants. The RNG is owned by
//! your own node's state (constructed once via [`rng_for_node`], kept across
//! the node's lifetime) - never re-seeded per call, or every election attempt
//! would draw the identical timeout and a split vote could repeat forever.
//!
//! Timing constants are generous relative to `turmoil`'s realistic simulated
//! latency (a `RequestVote`/`AppendEntries` round trip opens a fresh
//! connection per call in this module's own provided transport - a real
//! handshake-plus-round-trip cost, not free, even in simulated time) so that
//! correctly implementing Figure 2 doesn't also require discovering
//! connection-reuse-for-efficiency just to survive the module's own clock.
//! Simulated time is free to spend generously; there's no wall-clock cost to
//! these numbers being larger than a production Raft deployment would use.

use std::time::Duration;

pub const ELECTION_TIMEOUT_MIN_MS: u64 = 1000;
pub const ELECTION_TIMEOUT_MAX_MS: u64 = 2000;
pub const HEARTBEAT_INTERVAL_MS: u64 = 100;

/// A small, dependency-free, deterministic pseudo-random generator
/// (SplitMix64). Not from the `rand` crate: this crate's `Cargo.toml` only
/// reaches `rand` transitively through `turmoil`'s *dev*-dependency, never
/// available to library code (a real gap this project's own completion
/// roadmap already found once), and `rand::rngs::StdRng` explicitly does not
/// guarantee value-stability across its own versions - using it here would
/// silently break replay determinism on a future `rand` upgrade, exactly the
/// kind of seed-meaning drift this project pins `turmoil`'s own version
/// against. This generator's algorithm is a fixed, public, well-known
/// constant - its output for a given seed never changes.
#[derive(Debug, Clone, Copy)]
pub struct DeterministicRng(u64);

impl DeterministicRng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

/// Call once, at construction, seeded from the simulation's own seed plus
/// your node's own index - a stable mix, not a naive XOR (which correlates
/// too easily for small integers). Store the returned `DeterministicRng` in
/// your own state and keep drawing from the same instance for the life of
/// the node.
pub fn rng_for_node(sim_seed: u64, node_index: u32) -> DeterministicRng {
    let mut seed = sim_seed ^ 0x9E37_79B9_7F4A_7C15;
    seed = seed.wrapping_add((node_index as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    DeterministicRng(seed)
}

/// Draws one fresh, randomized election-timeout duration from your own owned
/// RNG - call this every time you schedule a new election timer (including
/// every retry after a split vote), never once and reused.
pub fn next_election_timeout(rng: &mut DeterministicRng) -> Duration {
    let span = ELECTION_TIMEOUT_MAX_MS - ELECTION_TIMEOUT_MIN_MS + 1;
    let ms = ELECTION_TIMEOUT_MIN_MS + (rng.next_u64() % span);
    Duration::from_millis(ms)
}

pub fn heartbeat_interval() -> Duration {
    Duration::from_millis(HEARTBEAT_INTERVAL_MS)
}
