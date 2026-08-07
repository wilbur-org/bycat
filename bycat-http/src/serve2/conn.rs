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

use crate::serve::Socket;
pin_project! {
    pub struct HyperConn<T: ?Sized> {
       #[pin]
       pub conn: T,
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
