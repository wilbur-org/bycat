use http::{Request, Response};
use hyper::{
    body::{Body, Incoming},
    service::{HttpService, Service},
};
use hyper_util::server::conn::auto::HttpServerConnExec;
use pin_project_lite::pin_project;
use std::{error::Error, pin::Pin, task::Poll};
use tokio::sync::watch;

use crate::server::Socket;

pub trait GracefulShutdown: Future<Output = Result<(), Self::Error>> {
    type Error;

    fn graceful_shutdown(self: Pin<&mut Self>);
}

impl<I, B, S> GracefulShutdown for hyper::server::conn::http1::Connection<I, S>
where
    S: HttpService<Incoming, ResBody = B>,
    S::Error: Into<Box<dyn Error + Send + Sync>>,
    I: Socket + Unpin + 'static,
    B: Body + 'static,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    type Error = hyper::Error;

    fn graceful_shutdown(self: Pin<&mut Self>) {
        hyper::server::conn::http1::Connection::graceful_shutdown(self);
    }
}

impl<I, B, S> GracefulShutdown for hyper::server::conn::http1::UpgradeableConnection<I, S>
where
    S: HttpService<Incoming, ResBody = B>,
    S::Error: Into<Box<dyn Error + Send + Sync>>,
    I: Socket + Send + Unpin + 'static,
    B: Body + 'static,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
{
    type Error = hyper::Error;

    fn graceful_shutdown(self: Pin<&mut Self>) {
        hyper::server::conn::http1::UpgradeableConnection::graceful_shutdown(self);
    }
}

impl<I, B, S, E> GracefulShutdown for hyper_util::server::conn::auto::Connection<'_, I, S, E>
where
    S: Service<Request<Incoming>, Response = Response<B>>,
    S::Error: Into<Box<dyn Error + Send + Sync>>,
    S::Future: 'static,
    I: Socket + Unpin + 'static,
    B: Body + 'static,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
    E: HttpServerConnExec<S::Future, B>,
{
    type Error = Box<dyn Error + Send + Sync>;

    fn graceful_shutdown(self: Pin<&mut Self>) {
        hyper_util::server::conn::auto::Connection::graceful_shutdown(self);
    }
}

impl<I, B, S, E> GracefulShutdown
    for hyper_util::server::conn::auto::UpgradeableConnection<'_, I, S, E>
where
    S: Service<Request<Incoming>, Response = Response<B>>,
    S::Error: Into<Box<dyn Error + Send + Sync>>,
    S::Future: 'static,
    I: Socket + Send + Unpin + 'static,
    B: Body + 'static,
    B::Error: Into<Box<dyn Error + Send + Sync>>,
    E: HttpServerConnExec<S::Future, B>,
{
    type Error = Box<dyn Error + Send + Sync>;

    fn graceful_shutdown(self: Pin<&mut Self>) {
        hyper_util::server::conn::auto::UpgradeableConnection::graceful_shutdown(self);
    }
}

pub struct Shutdown {
    sx: watch::Sender<()>,
}

impl Shutdown {
    pub fn new() -> Shutdown {
        let (sx, _) = watch::channel(());
        Shutdown { sx }
    }

    pub fn watch<C: GracefulShutdown>(&self, conn: C) -> impl Future<Output = C::Output> {
        let mut rx = self.sx.subscribe();

        GracefulWatchFuture {
            conn,
            cancel: async move {
                let _ = rx.changed().await;

                // hold onto the rx until the watched future is completed

                rx
            },
            guard: None,
        }
    }

    pub async fn shutdown(&self) {
        // signal all the watched futures about the change
        let _ = self.sx.send(());

        // and then wait for all of them to complete
        self.sx.closed().await;
    }

    pub(crate) fn clone(&self) -> Self {
        Self {
            sx: self.sx.clone(),
        }
    }
}

pin_project! {
  pub struct GracefulWatchFuture<C: GracefulShutdown, F: Future> {
    #[pin]
    conn: C,
    #[pin]
    cancel: F,
    #[pin]
    guard: Option<F::Output>
  }
}

impl<C, F> Future for GracefulWatchFuture<C, F>
where
    C: GracefulShutdown,
    F: Future,
{
    type Output = C::Output;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();

        if this.guard.is_none() {
            if let Poll::Ready(guard) = this.cancel.poll(cx) {
                this.guard.set(Some(guard));

                this.conn.as_mut().graceful_shutdown();
            }
        }

        this.conn.poll(cx)
    }
}
