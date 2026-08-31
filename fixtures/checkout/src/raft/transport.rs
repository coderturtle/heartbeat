//! Provided transport: wire framing (reusing Module 01's
//! `crate::rpc::{read_framed, write_framed}`) plus one-RPC-at-a-time exchange
//! for each Raft message kind.
//!
//! Ownership split: the test harness owns the accept loop (spawning
//! [`serve_one_rpc`] per accepted connection) and the peer *address book* -
//! which `NodeId` lives at which address, topology knowledge only the test
//! setup has, handed to you as the `peers: BTreeMap<NodeId, String>` your own
//! constructor receives, and which excludes your own id (cluster size is
//! `peers.len() + 1`). You own the actual decision to dial a specific peer -
//! calling `connector.connect(addr)` yourself, through the provided
//! `Connector`, whenever your own election or heartbeat logic decides to
//! contact someone. This module's exercise is entirely in `node.rs`: what a
//! node *does* with a message, not how bytes get there or who owns which
//! address.
//!
//! Every function here takes its stream *by value*, not `&mut` - so you can
//! `tokio::spawn` a call, or run several concurrently via `join_all`, without
//! a `'static` lifetime problem either way. Neither function has a built-in
//! timeout: wrap a call in `tokio::time::timeout(..)` yourself if you don't
//! want one slow or partitioned peer to block your own progress - see
//! `node.rs`'s module doc for why this is deliberately left to you, and note
//! that under a partition it's `connector.connect(addr)` itself that hangs,
//! not just the calls below.

use super::types::{
    AppendEntriesArgs, AppendEntriesReply, InboundMessage, InboundReply, RequestVoteArgs,
    RequestVoteReply,
};
use crate::rpc::{read_framed, write_framed};
use std::future::Future;
use tokio::io::{AsyncRead, AsyncWrite};

pub async fn call_request_vote<S>(
    mut stream: S,
    args: &RequestVoteArgs,
) -> std::io::Result<RequestVoteReply>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    write_framed(&mut stream, &InboundMessage::RequestVote(args.clone())).await?;
    match read_framed(&mut stream).await? {
        InboundReply::RequestVote(reply) => Ok(reply),
        _ => Err(std::io::Error::other("peer replied with the wrong RPC kind")),
    }
}

pub async fn call_append_entries<S>(
    mut stream: S,
    args: &AppendEntriesArgs,
) -> std::io::Result<AppendEntriesReply>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    write_framed(&mut stream, &InboundMessage::AppendEntries(args.clone())).await?;
    match read_framed(&mut stream).await? {
        InboundReply::AppendEntries(reply) => Ok(reply),
        _ => Err(std::io::Error::other("peer replied with the wrong RPC kind")),
    }
}

/// Reads one inbound message from `stream` and computes a reply via
/// `handler` - the harness's own accept loop spawns one call of this per
/// accepted connection so concurrent inbound RPCs don't block each other.
/// Not something you call yourself; documented here because it's the other
/// half of the wire contract your `handle_request_vote`/`handle_append_entries`
/// methods serve.
pub async fn serve_one_rpc<S, F, Fut>(mut stream: S, handler: F) -> std::io::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
    F: FnOnce(InboundMessage) -> Fut,
    Fut: Future<Output = InboundReply>,
{
    let msg: InboundMessage = read_framed(&mut stream).await?;
    let reply = handler(msg).await;
    write_framed(&mut stream, &reply).await
}
