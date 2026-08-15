mod builder;
mod conn;
pub mod server;

use bycat_executor::{LocalTokioExecutor, TokioExecutor};
use bycat_service::Work;
use http::Request;
use hyper::{body::Incoming, service::service_fn};
use tokio::net::{TcpListener, ToSocketAddrs};

use crate::{Error, IntoResponse, body::Body};

pub use self::builder::Builder;

#[cfg(feature = "serve-tokio")]
#[derive(Debug, Clone)]
pub struct Tokio<T> {
    builder: Builder,
    worker: T,
}

#[cfg(feature = "serve-tokio")]
impl<T> Tokio<T> {
    pub fn new(worker: T) -> Self {
        Self {
            builder: Builder::new(),
            worker,
        }
    }

    pub fn http1<F: FnOnce(&mut hyper::server::conn::http1::Builder)>(mut self, f: F) -> Self {
        f(self.builder.http1());
        self
    }

    pub fn upgradable(mut self, upgrade: bool) -> Self {
        self.builder.upgradable(upgrade);
        self
    }

    pub async fn serve<C>(self, addr: impl ToSocketAddrs, ctx: C) -> Result<(), Error>
    where
        T: Work<C, Request<Incoming>> + Clone + Send + 'static,
        T::Error: Into<Error>,
        T::Output: IntoResponse<Body>,
        for<'a> T::Future<'a>: Send,
        C: Clone + Send + 'static,
    {
        let listener = TcpListener::bind(addr).await.map_err(Error::custom)?;

        let worker = self.worker.clone();
        self.builder
            .listen(
                TokioExecutor,
                listener,
                service_fn(move |req: Request<_>| {
                    let worker = worker.clone();
                    let ctx = ctx.clone();
                    async move {
                        //
                        let resp = worker.call(&ctx, req).await.map_err(Into::into)?;
                        Ok(resp.into_response())
                    }
                }),
            )
            .await;

        Ok(())
    }

    pub async fn serve_local<C>(self, addr: impl ToSocketAddrs, ctx: C) -> Result<(), Error>
    where
        T: Work<C, Request<Incoming>> + Clone + 'static,
        T::Error: Into<Error>,
        T::Output: IntoResponse<Body>,
        C: Clone + 'static,
    {
        let listener = TcpListener::bind(addr).await.map_err(Error::custom)?;

        let worker = self.worker.clone();
        self.builder
            .listen(
                LocalTokioExecutor,
                listener,
                service_fn(move |req: Request<_>| {
                    let worker = worker.clone();
                    let ctx = ctx.clone();
                    async move {
                        //
                        let resp = worker.call(&ctx, req).await.map_err(Into::into)?;
                        Ok(resp.into_response())
                    }
                }),
            )
            .await;

        Ok(())
    }
}
