mod listener;
mod servable;
use self::servable::*;

use ::bycat_service::Service;
use bycat_executor::{LocalTokioExecutor, TokioExecutor};
use bycat_server::Shutdown;
use futures::future::BoxFuture;
use tokio::net::ToSocketAddrs;

#[derive(Debug, Clone)]
pub struct Tokio<T> {
    work: T,
    upgrade: bool,
}

impl<T> Tokio<T> {
    pub fn new(work: T) -> Self {
        Self {
            work,
            upgrade: false,
        }
    }

    pub fn upgradable(mut self, upgrade: bool) -> Self {
        self.upgrade = upgrade;
        self
    }

    pub async fn serve<C>(self, ctx: C, addr: impl ToSocketAddrs) -> Result<(), tokio::io::Error>
    where
        T: Service<
                C,
                http::Request<crate::body::Body>,
                Output = http::Response<crate::body::Body>,
                Error = crate::Error,
            >
            + Clone
            + 'static
            + Send,
        for<'a> T::Future<'a>: Send,
        C: Send + Clone + 'static,
    {
        let server = Server::new(TokioExecutor::default(), TokioServer(self.work, ctx))
            .with_upgrade(self.upgrade);

        let listener = tokio::net::TcpListener::bind(addr).await?;

        server.serve(listener, &Shutdown::new()).await;

        Ok(())
    }

    pub async fn serve_local<C>(
        self,
        ctx: C,
        addr: impl ToSocketAddrs,
    ) -> Result<(), tokio::io::Error>
    where
        T: Service<
                C,
                http::Request<crate::body::Body>,
                Output = http::Response<crate::body::Body>,
                Error = crate::Error,
            > + Clone
            + 'static,
        C: Clone + 'static,
    {
        let server = Server::new(LocalTokioExecutor::default(), TokioServer(self.work, ctx))
            .with_upgrade(self.upgrade);

        let listener = tokio::net::TcpListener::bind(addr).await?;

        server.serve(listener, &Shutdown::new()).await;

        Ok(())
    }
}
