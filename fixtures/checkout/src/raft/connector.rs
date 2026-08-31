//! The one piece of "networking" you initiate yourself: opening an outbound
//! connection to a peer. Generic so the same node logic runs against a real
//! `tokio::net::TcpStream` or a `turmoil::net::TcpStream` under test -
//! matching the precedent Module 01's own test file already established
//! (calling `TcpStream::connect` directly is test/deployment setup, not
//! "networking code" this workshop's fixtures scope out). `TokioConnector`
//! exists to make this trait's shape concrete and to compile - this
//! fixture's only real consumer is the test harness, not a production
//! deployment.

use tokio::io::{AsyncRead, AsyncWrite};

pub trait Connector: Clone + Send + Sync + 'static {
    type Stream: AsyncRead + AsyncWrite + Unpin + Send + 'static;

    fn connect(
        &self,
        addr: String,
    ) -> impl std::future::Future<Output = std::io::Result<Self::Stream>> + Send;
}

#[derive(Clone)]
pub struct TokioConnector;

impl Connector for TokioConnector {
    type Stream = tokio::net::TcpStream;

    async fn connect(&self, addr: String) -> std::io::Result<Self::Stream> {
        tokio::net::TcpStream::connect(addr).await
    }
}
