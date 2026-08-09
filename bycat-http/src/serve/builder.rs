use bycat_executor::Executor;
use bycat_service::Shutdown;
use http::{Request, Response};
use hyper::body::Incoming;

use crate::{
    Error,
    error::BoxError,
    serve::Listener,
    serve2::server::{Server, ServerFuture},
};

pub struct Builder<E> {
    executor: E,
    http1: hyper::server::conn::http1::Builder,
    shutdown: Option<Shutdown>,
}

impl<E> Builder<E> {
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            http1: hyper::server::conn::http1::Builder::new(),
            shutdown: None,
        }
    }

    pub fn set_shutdown(&mut self, shutdown: Shutdown) {
        self.shutdown = Some(shutdown);
    }

    pub fn http1(&mut self) -> &mut hyper::server::conn::http1::Builder {
        &mut self.http1
    }

    pub fn executor(&self) -> &E {
        &self.executor
    }

    pub async fn listen<L, W, B>(self, listener: L, service: W)
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

        Server::new(listener, self.executor, self.http1, service, shutdown)
            .serve()
            .await;
    }
}
