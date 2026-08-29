//! Module 01: RPC Over an Unreliable Network.
//!
//! `Checkout`'s own message shapes, and the RPC exchange that carries them.
//! No real checkout logic exists yet - this module only proves the RPC layer
//! can carry `CheckoutRequest`/`CheckoutResponse` faithfully over a stream
//! that a hostile simulated network can delay, drop, reorder, or partition
//! out from under it.
//!
//! `send_request` and `handle_one` are stubbed below (`todo!`). Implement
//! them against the provided test suite
//! (`tests/module_01_rpc_harness.rs`) - that suite is the deterministic
//! gate, not a spec to read and reimplement from prose.

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The domain type every later `Checkout`-facing module builds on. No real
/// checkout logic reads or writes these yet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckoutRequest {
    pub resource: String,
    pub holder: String,
    pub lease_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckoutResponse {
    Granted { generation: u64 },
    Denied,
}

/// Send one request over `stream` and return the one response that comes
/// back. `stream` behaves like a socket (it's a real `TcpStream` in
/// production, a `turmoil::net::TcpStream` under test) - this function must
/// not assume anything about the stream beyond `AsyncRead + AsyncWrite`.
///
/// Required behavior (see `tests/module_01_rpc_harness.rs` for the exact
/// checks): a request sent through a healthy connection gets exactly the
/// matching response back, unmodified. A connection that closes, times out,
/// or errors mid-exchange must surface as an `Err`, never a hang and never a
/// panic.
pub async fn send_request<S>(
    stream: &mut S,
    request: &CheckoutRequest,
) -> std::io::Result<CheckoutResponse>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    todo!("implement per this file's doc comments and tests/module_01_rpc_harness.rs")
}

/// Read exactly one request from `stream`, compute a response by calling
/// `handler` with it, and write that response back. Used by the exercise's
/// test server to answer one connection; a real multi-request server (not
/// this module's job) would call this in a loop per connection.
///
/// Required behavior: if the stream closes before a complete request
/// arrives (e.g. a `turmoil` partition fires mid-message), return an `Err`
/// rather than panicking or hanging - a partial message is not a request.
pub async fn handle_one<S>(
    stream: &mut S,
    handler: impl FnOnce(CheckoutRequest) -> CheckoutResponse,
) -> std::io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    todo!("implement per this file's doc comments and tests/module_01_rpc_harness.rs")
}

/// Write `value` to `stream` as a length-prefixed JSON frame: a 4-byte
/// big-endian length, then that many bytes of JSON body. TCP is a byte
/// stream, not a message stream - without a frame boundary, a receiver
/// can't tell where one message ends and the next begins.
///
/// Provided, not part of the exercise: correct framing is assumed
/// infrastructure here so the exercise stays scoped to the RPC exchange
/// itself, not wire-format design.
pub(crate) async fn write_framed<S, T>(stream: &mut S, value: &T) -> std::io::Result<()>
where
    S: AsyncWrite + Unpin,
    T: Serialize,
{
    let body = serde_json::to_vec(value).map_err(std::io::Error::other)?;
    let len = u32::try_from(body.len())
        .map_err(|_| std::io::Error::other("message too large to frame"))?;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&body).await?;
    stream.flush().await
}

/// The read-side counterpart to [`write_framed`]. Returns an `Err` if the
/// stream closes before a complete frame (length prefix or body) arrives.
pub(crate) async fn read_framed<S, T>(stream: &mut S) -> std::io::Result<T>
where
    S: AsyncRead + Unpin,
    T: for<'de> Deserialize<'de>,
{
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body).await?;
    serde_json::from_slice(&body).map_err(std::io::Error::other)
}
