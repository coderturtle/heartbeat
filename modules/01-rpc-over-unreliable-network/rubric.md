# Module 01 Rubric: RPC Over an Unreliable Network

Shared with the learner before the attempt, per Coachgremlin's own workflow. Criteria are property-phrased (an observable fact about the finished code), not technique-named (the fix itself stated outright).

| # | Criterion | Tier |
|---|---|---|
| 1 | `cargo test --test module_01_rpc_harness` is green, across every seed in the published set. | Gate, deterministic |
| 2 | `cargo clippy --tests -- -D warnings` is clean. | Gate, deterministic |
| 3 | The diff touches only `src/rpc.rs` - not `tests/module_01_rpc_harness.rs`. | Gate, anti-gaming |
| 4 | Every read from the stream either fills the requested buffer completely or reports how many bytes actually arrived - no read result is treated as "the whole message" without that check. | Scored, conceptual |
| 5 | `send_request` and `handle_one` make no assumption about the stream beyond `AsyncRead + AsyncWrite + Unpin` - nothing in `src/rpc.rs` imports `turmoil` directly or otherwise only works because the stream happens to be a simulated one. | Scored, conceptual |

## Why criterion 4 is scored, not gated

A real, evidenced finding from this module's own dry run (`runs/2026-08-29-module-01-dry-run/`): a naive implementation that uses `stream.read(&mut buf)` instead of `stream.read_exact(&mut buf)` - a real bug, since a single `read()` call on a TCP-like stream is never guaranteed to fill the whole buffer - passed all 5 of this module's `cargo test` cases identically to the correct implementation. Neither injected latency, a constrained `tcp_capacity`, nor a 200KB payload forced turmoil to actually split the naive implementation's read across multiple calls in this dry run's configuration. **`cargo clippy` did catch it instantly**, via its deny-by-default `unused_io_amount` lint - which is why criterion 2 (clippy clean) is listed as a separate gate criterion from criterion 1 (tests green), not folded into it. Criterion 4 exists so a learner who somehow silences or routes around that lint (e.g., an `#[allow]`) doesn't get a free pass - Coachgremlin checks the property directly, not just clippy's absence of complaint.

## What "explain why" means for criterion 4

Not "clippy told me to use `read_exact`." The learner should be able to state, in their own words, why TCP is a byte stream with no message boundaries of its own - so a `read()` call filling less of the buffer than requested is a real, spec-legal outcome, not a turmoil-specific quirk this dry run just didn't happen to trigger.
