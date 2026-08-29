# Retro: Heartbeat's first real content-building dry run

Module 01 is the first module of any Hekton workshop to move from skeleton to real, tested content since `Checkout`'s reseed and its own three-cycle doubt-driven-development hardening. This retro checks what that design work actually bought once real code existed to test it against.

## Did the DDD-hardened gate design hold up against real code?

**Partially, and the gap is itself the finding.** `docs/workshop-design.md`'s multi-seed, two-tier gate design assumed the deterministic tier's main risk was a test suite passing "by luck" on a narrow fault schedule. Module 01's real dry run found a different, real gap in the same tier: `cargo test` (even across 5 seeds and 5 distinct fault scenarios) didn't mechanically distinguish `read()` from `read_exact()` at all - not narrowly, not by luck, just didn't. The design's own two-tier vocabulary (deterministic primary, conceptual secondary) survived, but this dry run is evidence the *deterministic* tier itself has more than one mechanism inside it (tests and lints), and they don't cover the same bug classes. Worth naming explicitly for Coachgremlin's future modules: don't assume `cargo test` alone is "the deterministic tier" - `cargo clippy` is doing real, independent, sometimes load-bearing work.

## Did turmoil actually earn its place here?

**Yes, for the fault classes this module's tests targeted (latency, partition, dropped connections) - all three), all four of those scenarios discriminate correctly and are exactly the kind of network-condition-specific bug a plain `cargo test` without any fault injection would never have exercised at all.** The short-read gap doesn't undercut that; it's a different question (byte-level delivery granularity) that neither `turmoil`'s current configuration knobs nor a larger payload happened to expose in this dry run.

## What Coachgremlin's Module 02 dry run should check fresh, not assume

Per this workshop's own discipline (`docs/decisions.md`'s doubt-driven-development entries all show finding-per-cycle, not clean-pass-per-cycle): don't assume Module 01's specific "clippy catches what tests don't" split generalizes. Module 02 (the single-node `Checkout` service - exclusivity, lease expiry, generation fencing) has a genuinely different bug surface (concurrency/state correctness, not wire-level framing) where the opposite might hold, or a real conceptual-tier finding might be needed for the first time in this workshop. Check fresh.

## Go/no-go on Module 01

**Go.** Rubric met by a real correct attempt, a real naive attempt correctly fails a gate criterion, and the reason it fails is understood, not just observed. Takeaway (`turmoil`-based fault-injection harness template) is real and demonstrated, not aspirational.
