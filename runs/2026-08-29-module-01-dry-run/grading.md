# Coachgremlin's dry run: Module 01, RPC Over an Unreliable Network

First real content-building dry run for Heartbeat, per Coachgremlin's own evidenced practice (`borrow-native`'s Module 01-03 dry runs): a correct attempt and a deliberately naive, honest (non-adversarial) attempt, both against the same stubbed exercise (`fixtures/checkout/src/rpc.rs`'s `send_request`/`handle_one`), both run against the same provided test suite (`fixtures/checkout/tests/module_01_rpc_harness.rs`).

## Step 3: Observe the attempt

**`attempt-good/`** (`diff.patch`): implements `send_request`/`handle_one` by delegating to the provided `write_framed`/`read_framed` helpers, which use `read_exact`/`write_all` throughout. `cargo test --test module_01_rpc_harness`: 5/5 pass, across all 5 published seeds each. `cargo clippy --tests -- -D warnings`: clean.

**`attempt-naive-short-read/`** (`diff.patch`): identical `send_request`/`handle_one`, but `read_framed` uses `stream.read(&mut buf)` instead of `stream.read_exact(&mut buf)` for both the length prefix and the body - a real, honest mistake (not an adversarial one): `read()` on an async stream is never guaranteed to fill the whole buffer in one call, a very plausible thing for someone hand-rolling a wire protocol for the first time to get wrong.

## Step 3 (continued): what turmoil's fault injection did and didn't catch

**`cargo test` passed 5/5 identically on both attempts** - including the latency-injection test. Before accepting that as the finding, three escalating probes tried to force turmoil to actually split the naive implementation's read across multiple `read()` calls, since that's exactly the condition that would expose the bug mechanically:

1. Baseline (no extra config): passed.
2. `.tcp_capacity(1)` (constrain the simulated TCP buffer to 1 byte): still passed.
3. `.tcp_capacity(64)` plus a 200KB request body (`"x".repeat(200_000)` in the `resource` field): still passed.

None of the three forced a short read in this dry run's configuration. **This is a real, evidenced finding, not a gap in the probes tried**: this module's `cargo test` suite, as currently designed, cannot mechanically distinguish `read()` from `read_exact()` for this exercise's message sizes, at least not via `turmoil`'s latency/capacity knobs alone. Recorded honestly in `modules/01-rpc-over-unreliable-network/rubric.md` rather than assumed away.

**`cargo clippy --tests -- -D warnings` caught it instantly** on the naive attempt - two hard errors, `unused_io_amount` (`clippy::unused_io_amount` is deny-by-default in Clippy, not merely warn), pointing at both `read()` calls in `read_framed` and suggesting `read_exact` by name. This is why the rubric (see `modules/01-rpc-over-unreliable-network/rubric.md`) lists "tests green" and "clippy clean" as two separate gate criteria rather than folding them into one "deterministic tier" line - for this specific bug shape, they are not redundant checks of the same thing.

## Step 4: Score against the rubric

| # | Criterion | attempt-good | attempt-naive-short-read |
|---|---|---|---|
| 1 | `cargo test` green (gate, deterministic) | Pass. 5/5, all seeds. | **Pass.** 5/5, all seeds - the deterministic test suite alone does not catch this bug. |
| 2 | `cargo clippy --tests -- -D warnings` clean (gate, deterministic) | Pass. | **Fails.** Two `unused_io_amount` errors. |
| 3 | Diff touches only `src/rpc.rs` (gate, anti-gaming) | Pass. | Pass. |
| 4 | Every read either fills the buffer or reports how many bytes arrived (scored, conceptual) | Pass. | **Fails**, redundantly with criterion 2 - see "why criterion 4 is scored, not gated" in the rubric for why this redundancy is deliberate, not wasted. |
| 5 | No stream assumption beyond `AsyncRead + AsyncWrite + Unpin`, no `turmoil` leakage (scored, conceptual) | Pass. | Pass - the bug is orthogonal to this criterion. |

**Result: `cargo clippy` alone was sufficient to separate the two attempts here** - a materially different outcome from `borrow-native`'s own Module 01/02/03 dry runs, where neither `cargo test` nor default `cargo clippy` distinguished a naive attempt from a correct one, and only the conceptual tier (or, for Module 01 specifically, `clippy::pedantic`) caught anything. This module's real finding is the mirror image: the *test suite* is the weak tier here, not clippy.

## Step 5: Confirm or loop

- **attempt-good:** rubric met.
- **attempt-naive-short-read:** rubric not met (gate criterion 2 fails outright - this attempt would not advance).

## Takeaway

Packaged: a `turmoil`-based network-fault-injection harness template (`fixtures/checkout/tests/module_01_rpc_harness.rs` itself, once genericized past this one exercise - Coachgremlin's next real content pass, not this dry run's job).

## What this dry run is and isn't evidence of

**Is:** confirmation that Module 01's exercise, test suite, and rubric are real and internally consistent - a correct attempt passes everything, a real, honest wrong attempt fails at least one gate criterion, and the *reason* it fails is understood and documented rather than assumed.

**Is also:** a genuine, first-of-its-kind finding for this workshop, distinct in shape from every `borrow-native` dry-run finding to date: there, the risk was "the deterministic tier can't tell correct from lucky, only the conceptual tier can." Here, one half of the deterministic tier (`cargo test`, even under real fault injection) couldn't tell them apart, but the *other* half of the same deterministic tier (`cargo clippy`) could - meaning the fix wasn't "add a conceptual criterion," it was "recognize both checks as necessary, independent gates." Worth watching whether this "some fault classes need clippy, not turmoil" pattern recurs in Modules 02+ before treating it as this workshop's general story, the same discipline `borrow-native` used before generalizing its own findings.

**Isn't:** evidence that `turmoil`'s fault injection is weak in general - only that this specific bug shape (a short read within message sizes this dry run tried) isn't one of the fault classes `latency`/`tcp_capacity` reliably exercise. A different bug shape (e.g. this module's own partition/dropped-connection tests) is exactly the kind of thing `turmoil` *did* catch correctly on both attempts (both attempts pass those tests identically, correctly, since the bug is orthogonal to partition handling).

**Isn't:** a new, independent data point toward Coachgremlin's 3-run Review Trigger (that bar counts distinct workshops, not modules within one) - but it is real, first-time evidence for *this* workshop's own two-tier design actually working end to end on real code, not just design-doc prose.
