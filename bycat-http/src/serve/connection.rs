use alloc::boxed::Box;
use bycat_service::{GracefulShutdown, Shutdown};
use core::error::Error as StdError;
use http::{Request, Response};
use hyper::{
    body::{Body, Incoming},
    server::conn::http1::Builder,
    service::{HttpService, Service},
};
use pin_project_lite::pin_project;

use super::{Listener, listener::Socket};

// #[derive(Debug, Clone, Default)]
// pub struct LocalTokioExecutor;

// impl<T> Executor<T> for LocalTokioExecutor
// where
//     T: Future + 'static,
// {
//     fn execute(&self, fut: T) {
//         tokio::task::spawn_local(fut);
//     }
// }

pub struct Connection<L, E>
where
    L: Listener,
{
    builder: Builder,
    shutdown: Shutdown,
    socket: L::Io,
    local_address: L::Addr,
    // Should be used for http2
    #[allow(unused)]
    executor: E,
}

impl<L, E> Connection<L, E>
where
    L: Listener,
{
    pub(crate) fn new(
        executor: E,
        builder: Builder,
        shutdown: Shutdown,
        socket: L::Io,
        local_address: L::Addr,
    ) -> Connection<L, E> {
        Connection {
            builder,
            local_address,
            shutdown,
            socket,
            executor,
        }
    }
}

impl<L, E> Connection<L, E>
where
    L: Listener,
{
    pub fn local_address(&self) -> &L::Addr {
        &self.local_address
    }

    pub fn socket(&self) -> &L::Io {
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
        L::Io: 'static,
    {
        let conn = self.builder.serve_connection(self.socket, service);
        self.shutdown
            .watch(HyperConn { conn })
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
        L::Io: Send + 'static,
        // I: Socket + Unpin + Send + 'static,
    {
        let conn = self
            .builder
            .serve_connection(self.socket, service)
            .with_upgrades();
        self.shutdown
            .watch(HyperConn { conn })
            .await
            .map_err(|e| Box::new(e) as _)
    }
}

pin_project! {
    struct HyperConn<T: ?Sized> {
       #[pin]
       conn: T,
    }
}

impl<T> Future for HyperConn<T>
where
    T: Future,
{
    type Output = T::Output;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        self.project().conn.poll(cx)
    }
}

impl<I, B, S> GracefulShutdown for HyperConn<hyper::server::conn::http1::Connection<I, S>>
where
    S: HttpService<Incoming, ResBody = B>,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
    I: Socket + Unpin + 'static,
    B: Body + 'static,
    B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    type Error = hyper::Error;
    fn graceful_shutdown(self: core::pin::Pin<&mut Self>) {
        hyper::server::conn::http1::Connection::graceful_shutdown(self.project().conn);
    }
}

impl<I, B, S> GracefulShutdown
    for HyperConn<hyper::server::conn::http1::UpgradeableConnection<I, S>>
where
    S: HttpService<Incoming, ResBody = B>,
    S::Error: Into<Box<dyn StdError + Send + Sync>>,
    I: Socket + Send + Unpin + 'static,
    B: Body + 'static,
    B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    type Error = hyper::Error;

    fn graceful_shutdown(self: core::pin::Pin<&mut Self>) {
        hyper::server::conn::http1::UpgradeableConnection::graceful_shutdown(self.project().conn);
    }
}
