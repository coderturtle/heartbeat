//! Module 01's deterministic gate: the RPC harness, under test.
//!
//! This file is provided, not the learner's job to write - it is what
//! `send_request`/`handle_one` (in `src/rpc.rs`) must satisfy. A published,
//! fixed set of seeds ships with this skeleton; Coachgremlin's real
//! content-authoring pass expands it to the full practice/held-out split
//! `docs/workshop-design.md` requires (>= 50 independently-generated seeds
//! per set, disjoint from each other) before this module is graded for
//! real.

use checkout::rpc::{handle_one, send_request, CheckoutRequest, CheckoutResponse};
use std::time::Duration;
use turmoil::net::{TcpListener, TcpStream};

const SERVER_ADDR: &str = "0.0.0.0:9000";

/// A small, published, illustrative seed set. Not the real practice/held-out
/// split - see this file's own doc comment above.
const SEEDS: &[u64] = &[1, 2, 3, 5, 8];

fn sample_request() -> CheckoutRequest {
    CheckoutRequest {
        resource: "example/repo:main".to_string(),
        holder: "agent-session-42".to_string(),
        lease_duration_ms: 30_000,
    }
}

/// The exercise's stub handler: always grants, generation 1. No real
/// `Checkout` logic exists yet in Module 01 - this module only proves the
/// RPC layer carries a request/response pair faithfully.
fn stub_handler(_req: CheckoutRequest) -> CheckoutResponse {
    CheckoutResponse::Granted { generation: 1 }
}

fn spawn_server(sim: &mut turmoil::Sim<'_>) {
    sim.host("server", || async {
        let listener = TcpListener::bind(SERVER_ADDR).await?;
        loop {
            let (mut stream, _) = listener.accept().await?;
            // One request per connection is enough for this module's exercise.
            let _ = handle_one(&mut stream, stub_handler).await;
        }
    });
}

/// Baseline: a healthy connection with no injected faults gets exactly the
/// stub handler's response back, on every seed.
#[test]
fn healthy_exchange_round_trips() {
    for &seed in SEEDS {
        let mut sim = turmoil::Builder::new().rng_seed(seed).build();
        spawn_server(&mut sim);

        sim.client("client", async {
            let mut stream = TcpStream::connect("server:9000").await?;
            let response = send_request(&mut stream, &sample_request()).await?;
            assert_eq!(response, CheckoutResponse::Granted { generation: 1 });
            Ok(())
        });

        sim.run()
            .unwrap_or_else(|e| panic!("healthy exchange failed for seed {seed}: {e}"));
    }
}

/// Latency: a message delayed within a bounded window still arrives intact,
/// on every seed in the set.
#[test]
fn exchange_survives_injected_latency() {
    for &seed in SEEDS {
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .min_message_latency(Duration::from_millis(50))
            .max_message_latency(Duration::from_millis(500))
            .build();
        spawn_server(&mut sim);

        sim.client("client", async {
            let mut stream = TcpStream::connect("server:9000").await?;
            let response = send_request(&mut stream, &sample_request()).await?;
            assert_eq!(response, CheckoutResponse::Granted { generation: 1 });
            Ok(())
        });

        sim.run()
            .unwrap_or_else(|e| panic!("latency-tolerant exchange failed for seed {seed}: {e}"));
    }
}

/// A partition that never heals during the exchange must surface as a
/// connection error the caller can observe, never a silent hang - across
/// every seed.
#[test]
fn partition_surfaces_as_an_error_not_a_hang() {
    for &seed in SEEDS {
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(Duration::from_secs(10))
            .build();
        spawn_server(&mut sim);

        sim.client("client", async {
            turmoil::partition("client", "server");
            let connect = tokio::time::timeout(
                Duration::from_secs(5),
                TcpStream::connect("server:9000"),
            )
            .await;
            assert!(
                connect.is_err() || connect.unwrap().is_err(),
                "connecting across a standing partition should time out or fail, not silently succeed"
            );
            Ok(())
        });

        sim.run()
            .unwrap_or_else(|e| panic!("partition test failed for seed {seed}: {e}"));
    }
}

/// A partition that heals mid-exchange lets a subsequent request through -
/// across every seed.
#[test]
fn exchange_recovers_after_partition_heals() {
    for &seed in SEEDS {
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(Duration::from_secs(10))
            .build();
        spawn_server(&mut sim);

        sim.client("client", async {
            turmoil::partition("client", "server");
            let blocked = tokio::time::timeout(
                Duration::from_millis(500),
                TcpStream::connect("server:9000"),
            )
            .await;
            assert!(
                blocked.is_err() || blocked.unwrap().is_err(),
                "the partition should have blocked this connection attempt"
            );

            turmoil::repair("client", "server");
            let mut stream = TcpStream::connect("server:9000").await?;
            let response = send_request(&mut stream, &sample_request()).await?;
            assert_eq!(response, CheckoutResponse::Granted { generation: 1 });
            Ok(())
        });

        sim.run()
            .unwrap_or_else(|e| panic!("post-repair exchange failed for seed {seed}: {e}"));
    }
}

/// A connection dropped mid-frame (message loss) must surface as an error
/// on the client's next read, never a hang and never a value silently
/// materialized from a partial frame - across every seed.
#[test]
fn dropped_connection_mid_message_is_an_error_not_a_hang() {
    for &seed in SEEDS {
        let mut sim = turmoil::Builder::new()
            .rng_seed(seed)
            .simulation_duration(Duration::from_secs(10))
            .build();

        sim.host("server", || async {
            let listener = TcpListener::bind(SERVER_ADDR).await?;
            let (stream, _) = listener.accept().await?;
            // Accept the connection, then drop it without responding -
            // simulates a server crash mid-exchange.
            drop(stream);
            std::future::pending::<()>().await;
            #[allow(unreachable_code)]
            Ok(())
        });

        sim.client("client", async {
            let mut stream = TcpStream::connect("server:9000").await?;
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                send_request(&mut stream, &sample_request()),
            )
            .await;
            assert!(
                matches!(result, Ok(Err(_))) || result.is_err(),
                "a connection dropped mid-exchange must surface as an error, not a hang or a fabricated response"
            );
            Ok(())
        });

        sim.run()
            .unwrap_or_else(|e| panic!("dropped-connection test failed for seed {seed}: {e}"));
    }
}
