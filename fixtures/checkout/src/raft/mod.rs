//! Raft (Modules 03-06): leader election, log replication, persistence, and
//! log compaction/snapshots, built incrementally on the same types across all
//! four modules. `types`, `timer`, `connector`, and `transport` are provided
//! infrastructure; `node` is the exercise.

pub mod connector;
pub mod node;
pub mod timer;
pub mod transport;
pub mod types;
