use super::{Shutdown, Socket};
use http::{Request, Response};
use hyper::{
    body::{Body, Incoming},
    rt::Executor,
    server::conn::http1::Builder,
    service::Service,
};
use std::error::Error as StdError;

#[derive(Debug, Clone, Default)]
pub struct LocalTokioExecutor;

impl<T> Executor<T> for LocalTokioExecutor
where
    T: Future + 'static,
{
    fn execute(&self, fut: T) {
        tokio::task::spawn_local(fut);
    }
}

pub use hyper_util::rt::TokioExecutor;

pub struct Connection<I, A, E> {
    builder: Builder,
    shutdown: Shutdown,
    socket: I,
    local_address: A,
    // Should be used for http2
    #[allow(unused)]
    executor: E,
}

impl<I, A, E> Connection<I, A, E> {
    pub(crate) fn new(
        executor: E,
        builder: Builder,
        shutdown: Shutdown,
        socket: I,
        local_address: A,
    ) -> Connection<I, A, E> {
        Connection {
            builder,
            local_address,
            shutdown,
            socket,
            executor,
        }
    }
}

impl<I, A, E> Connection<I, A, E> {
    pub fn local_address(&self) -> &A {
        &self.local_address
    }

    pub fn socket(&self) -> &I {
        &self.socket
    }

    pub async fn serve_connection<S, B>(
        self,
        service: S,
    ) -> Result<(), Box<dyn StdError + Send + Sync + 'static>>
    where
        S: Service<Request<Incoming>, Response = Response<B>>,
        S::Error: Into<Box<dyn StdError + Send + Sync>>,
        B: Body + 'static,
        B::Error: Into<Box<dyn StdError + Send + Sync>>,
        I: Socket + Unpin + 'static,
    {
        let conn = self.builder.serve_connection(self.socket, service);
        self.shutdown
            .watch(conn)
            .await
            .map_err(|e| Box::new(e) as _)
    }

    pub async fn serve_connection_with_upgrades<S, B>(
        self,
        service: S,
    ) -> Result<(), Box<dyn StdError + Send + Sync + 'static>>
    where
        S: Service<Request<Incoming>, Response = Response<B>>,
        S::Error: Into<Box<dyn StdError + Send + Sync>>,
        B: Body + 'static,
        B::Error: Into<Box<dyn StdError + Send + Sync>>,
        I: Socket + Unpin + Send + 'static,
    {
        let conn = self
            .builder
            .serve_connection(self.socket, service)
            .with_upgrades();
        self.shutdown
            .watch(conn)
            .await
            .map_err(|e| Box::new(e) as _)
    }
}
