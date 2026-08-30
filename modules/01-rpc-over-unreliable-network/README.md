# Module 01: RPC Over an Unreliable Network

## The question this module answers

How do I build an RPC layer, and the harness that can lie to it convincingly?

## Where it sits in the arc

First module. No conceptual prerequisite - but see the callout below, this is not a warm-up. Next: [Module 02, A Single-Node Checkout Service](../02-single-node-checkout-service/README.md), which builds `Checkout`'s first real service on top of the RPC layer this module produces. See [`modules/README.md`](../README.md) for the full arc, why this order, and how this module's own exercise fits into `Checkout`, the one product this whole arc builds.

**This module is not a gentle warm-up.** It's where you build the `turmoil`-backed network-fault-injection harness every later module's deterministic gate depends on - real engineering work, with zero distributed-systems intuition yet to lean on. See `docs/workshop-design.md`'s callout on this, added after this workshop's own Review Panel flagged the "no prerequisite" framing as underselling the difficulty.

## Exercise: implement `send_request` and `handle_one`

Runs against `fixtures/checkout/`, the shared project every module in this workshop builds toward (see [`modules/README.md`](../README.md)'s "One shared project" section for the full build-out). Module 01 adds the RPC layer and `Checkout`'s own message shapes.

> Implement `send_request` and `handle_one` in `fixtures/checkout/src/rpc.rs` (both currently `todo!()`): a client-side function that sends one `CheckoutRequest` and returns the matching `CheckoutResponse`, and a server-side function that reads one request, computes a response via a supplied handler, and writes it back. Both must work over any stream implementing `AsyncRead + AsyncWrite + Unpin` - a real `TcpStream` in production, a `turmoil::net::TcpStream` under test - not just the one you happen to test against. Two framing helpers, `write_framed`/`read_framed`, are provided; use them rather than reinventing wire framing, which isn't this module's exercise. Get `cargo test --test module_01_rpc_harness` green and `cargo clippy --tests -- -D warnings` clean, from your own harness, without narrating the fix as you go, then check your diff against the rubric below.

## Rubric

See [`rubric.md`](rubric.md) for the full table and rationale. Five criteria: `cargo test` green (gate), `cargo clippy` clean (gate), diff touches only `src/rpc.rs` (gate, anti-gaming), every stream read either fills its buffer or reports how many bytes arrived (scored, conceptual), and no stream assumption beyond `AsyncRead + AsyncWrite + Unpin` (scored, conceptual).

**Before trusting a green `cargo test` alone:** this module's own dry run (`runs/2026-08-29-module-01-dry-run/grading.md`) found a real, honest implementation mistake - using `stream.read()` instead of `stream.read_exact()` - that passed all 5 provided tests identically to a correct implementation, across every published seed, even under injected latency and a deliberately oversized request. `cargo test` alone is not sufficient evidence you're done here; `cargo clippy` catches this specific mistake instantly (`unused_io_amount`, deny-by-default), which is exactly why it's listed as its own separate gate criterion, not folded into "tests pass."

## Required to advance / stop condition

Produce an implementation of `send_request` and `handle_one` that passes `cargo test --test module_01_rpc_harness` and `cargo clippy --tests -- -D warnings`, touches only `fixtures/checkout/src/rpc.rs`, and reads every byte it claims to have read. Reading this page does not count: you advance on a working implementation Coachgremlin has actually reviewed against the rubric above, not on having read it.

**Valid alternate terminal:** if your first working solution passes `cargo test` but fails `cargo clippy`, that's not a failure, it's the actual exercise - this module's own dry run found exactly that gap. Read what clippy is telling you, understand why a single `read()` call isn't guaranteed to fill your buffer, and fix the read loop rather than suppressing the lint.

## Learning objectives

- Build an async Rust RPC mechanism (request/response over a simulated network) from first principles, without reaching for an existing RPC framework.
- Use `turmoil` to inject latency, packet loss, reordering, and partitions into that RPC mechanism, and observe each fault's effect directly.
- Recognize that TCP is a byte stream with no message boundaries of its own - a `read()` call filling less of a buffer than requested is a real, spec-legal outcome, not a simulator quirk.
- Explain, from having built it, what MIT 6.5840's own `labrpc` package does for Go learners and why an equivalent doesn't yet exist off the shelf for Rust.

## Why this is hard, and what actually turned out to matter

**Don't read this section before your first attempt either** - it names the diagnosis directly. Attempt the exercise first; come back here once you have a working `cargo test` pass (or you're genuinely stuck on tooling, not the concept).

The obvious way to read a length-prefixed message is `stream.read(&mut buf)` once for the length, once for the body - and on a loopback-like connection with no real congestion, it usually just works, every time you try it locally. That's exactly what makes it a trap: `read()`'s contract has never promised to fill your buffer, only to fill *some* of it (or none, at EOF) - your local testing happened not to exercise the gap, the same thing this module's own dry run found when even `turmoil`'s injected latency and a 200KB payload didn't force a short read either. The fix isn't a special case for slow networks; it's using `read_exact` (or your own loop) everywhere, because "fill this buffer completely" is what you actually mean, and nothing about a byte stream promises it for free.

## Takeaway

A `turmoil`-based network-fault-injection harness template (`fixtures/checkout/tests/module_01_rpc_harness.rs`): reusable scaffolding for testing any future async Rust project's behavior under packet loss, latency, reordering, and partitions.

## Content status

**Real, not a placeholder.** Exercise, fixture, test suite, and rubric all exist and have been dry-run against a correct attempt and a real naive attempt (`runs/2026-08-29-module-01-dry-run/`). See [`modules/README.md`](../README.md) for workshop-wide status - Modules 02-09 are still skeleton only.
