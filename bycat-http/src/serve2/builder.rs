use bycat_executor::Executor;
use bycat_service::Shutdown;
use http::{Request, Response};
use hyper::body::Incoming;

use super::{server::Server, server::ServerFuture};
use crate::{Error, error::BoxError, serve::Listener};

#[derive(Debug, Clone)]
pub struct Builder {
    http1: hyper::server::conn::http1::Builder,
    shutdown: Option<Shutdown>,
    upgrade: bool,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            http1: hyper::server::conn::http1::Builder::new(),
            shutdown: None,
            upgrade: false,
        }
    }

    pub fn set_shutdown(&mut self, shutdown: Shutdown) {
        self.shutdown = Some(shutdown);
    }

    pub fn http1(&mut self) -> &mut hyper::server::conn::http1::Builder {
        &mut self.http1
    }

    pub fn upgradable(&mut self, upgrade: bool) {
        self.upgrade = upgrade;
    }

    pub async fn listen<L, W, B, E>(self, executor: E, listener: L, service: W)
    where
        E: Executor<ServerFuture<L, W, B>>,
        L: Listener,
        L::Io: 'static,
        W: hyper::service::Service<Request<Incoming>, Response = Response<B>, Error = Error>
            + Clone,
        B: http_body::Body + 'static,
        B::Error: Into<BoxError>,
    {
        let shutdown = self.shutdown.unwrap_or_else(|| Shutdown::new());

        Server::new(
            listener,
            executor,
            self.http1,
            service,
            shutdown,
            self.upgrade,
        )
        .serve()
        .await;
    }
}
