use crate::{
    Error, IntoResponse, body::HttpBody, error::BoxError, serve::Listener, serve2::conn::HyperConn,
};
use alloc::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use bycat_executor::Executor;
use bycat_futures::IntoResult;
use bycat_service::{GracefulWatchFuture, Shutdown};
use futures::{FutureExt, future::Either};
use http::{Request, Response};
use hyper::{
    body::Incoming,
    server::conn::http1::{Builder, UpgradeableConnection},
};
use pin_project_lite::pin_project;

pub type ServerFuture<L, W, B> = GracefulWatchFuture<
    HyperConn<hyper::server::conn::http1::Connection<<L as Listener>::Io, ServerService<W, B>>>,
>;

pub type UpgradeServerFuture<L, W, B> = GracefulWatchFuture<
    HyperConn<
        UpgradeableConnection<
            hyper::server::conn::http1::Connection<<L as Listener>::Io, ServerService<W, B>>,
            ServerService<W, B>,
        >,
    >,
>;

pub type UnifiedServerFuture<L, W, B> = Either<ServerFuture<L, W, B>, UpgradeServerFuture<L, W, B>>;

pin_project! {
    pub struct Server<L, E, W, B>

    {
        listener: L,
        executor: E,
        builder: Builder,
        service: W,
        shutdown: Shutdown,
        body: PhantomData<B>,
        upgrade: bool,
    }
}

impl<L, E, W, B> Server<L, E, W, B>
where
    L: Listener,
    L::Io: Send + 'static,
    W: hyper::service::Service<Request<Incoming>, Response = Response<B>, Error = Error> + Clone,
    B: http_body::Body + 'static,
    B::Error: Into<BoxError>,
    E: Executor<UpgradeServerFuture<L, W, B>>,
{
    pub fn new(
        listener: L,
        executor: E,
        builder: Builder,
        service: W,
        shutdown: Shutdown,
        upgrade: bool,
    ) -> Self {
        Server {
            listener,
            executor,
            builder,
            service,
            shutdown,
            body: PhantomData,
            upgrade,
        }
    }

    pub async fn serve(self) {
        let Server {
            mut listener,
            executor,
            builder,
            service,
            shutdown,
            ..
        } = self;

        let mut wait = shutdown.wait().fuse();

        loop {
            futures::select_biased! {
                next = listener.accept().fuse() => {
                    let (socket, _) = next;


                    let conn = builder.serve_connection(socket, ServerService { service: service.clone(), body: PhantomData });
                    if self.upgrade {
                        let conn = conn.with_upgrades();
                        let future = shutdown.watch(HyperConn { conn });
                        executor.spawn(Either::Right(future));

                    } else {
                        let future = shutdown.watch(HyperConn { conn });
                        executor.spawn(Either::Left(future));
                    }
                    // let conn = builder.serve_connection(socket, ServerService { service: service.clone(), body: PhantomData });
                    // let future = shutdown.watch(HyperConn { conn });

                    // executor.spawn(future);
                }
                _ = &mut wait => {
                    break;
                }
            };
        }
    }
}

pub struct ServerService<S, B> {
    service: S,
    body: PhantomData<B>,
}

unsafe impl<S: Send, B> Send for ServerService<S, B> {}

unsafe impl<S: Sync, B> Sync for ServerService<S, B> {}

impl<S, B> hyper::service::Service<Request<Incoming>> for ServerService<S, B>
where
    S: hyper::service::Service<Request<Incoming>> + Clone,
    S::Response: IntoResponse<B>,
    S::Error: Into<Error>,
    B: http_body::Body + 'static,
    B::Error: Into<BoxError>,
{
    type Response = Response<B>;
    type Error = Error;
    type Future = ServerServiceFuture<S::Future, B>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        ServerServiceFuture {
            future: self.service.call(req),
            body: PhantomData,
        }
    }
}

pin_project! {
    pub struct ServerServiceFuture<F, B> {
        #[pin]
        future: F,
        body: PhantomData<B>,
    }
}

impl<F, B> Future for ServerServiceFuture<F, B>
where
    F: Future,
    F::Output: IntoResult,
    <F::Output as IntoResult>::Output: IntoResponse<B>,
    <F::Output as IntoResult>::Error: Into<Error>,
    B: http_body::Body + 'static,
    B::Error: Into<BoxError>,
{
    type Output = Result<Response<B>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let output = futures::ready!(this.future.poll(cx)).into_result();
        match output {
            Ok(output) => Poll::Ready(Ok(output.into_response())),
            Err(err) => Poll::Ready(Err(err.into())),
        }
    }
}
