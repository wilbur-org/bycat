use bycat_service::Shutdown;
use futures::FutureExt;
use http::{Request, Response};
use http_body::Body;
use hyper::{body::Incoming, server::conn::http1::Builder, service::Service};

use crate::error::BoxError;

use super::{Connection, listener::Listener};

pub struct Conn<L, E>
where
    L: Listener,
{
    conn: Connection<L, E>,
    with_upgrade: bool,
}

impl<L, E> Conn<L, E>
where
    L: Listener,
{
    pub async fn serve_connection<S, B>(self, service: S) -> Result<(), BoxError>
    where
        S: Service<Request<Incoming>, Response = Response<B>>,
        S::Error: Into<BoxError>,
        B: Body + 'static,
        B::Error: Into<BoxError>,
        L::Io: Send + 'static,
    {
        if self.with_upgrade {
            self.conn.serve_connection_with_upgrades(service).await
        } else {
            self.conn.serve_connection(service).await
        }
    }
}

pub trait Servable<E, L>
where
    L: Listener,
{
    type Future<'a>: Future<Output = ()>
    where
        Self: 'a;

    fn call(&self, conn: Conn<L, E>) -> Self::Future<'_>;
}

pub struct Server<T, E> {
    service: T,
    builder: Builder,
    executor: E,
    with_upgrade: bool,
}

impl<T, E> Server<T, E> {
    pub fn new(executor: E, service: T) -> Server<T, E> {
        Server {
            service,
            builder: Builder::new(),
            executor,
            with_upgrade: false,
        }
    }

    pub fn with_upgrade(mut self, with_upgrade: bool) -> Self {
        self.with_upgrade = with_upgrade;
        self
    }
}

impl<T, E> Server<T, E>
where
    E: Clone,
{
    pub async fn serve<L>(&self, mut listener: L, shutdown: &Shutdown)
    where
        L: Listener,
        T: Servable<E, L>,
    {
        let inner = Shutdown::new();

        let mut wait = shutdown.wait().fuse();

        loop {
            futures::select_biased! {
                (stream, address) = listener.accept().fuse() => {

                    let conn = Connection::new(self.executor.clone(),self.builder.clone(), inner.clone(), stream, address);

                    self.service.call(Conn { conn, with_upgrade: self.with_upgrade }).await;
                }
                _ = &mut wait => {
                    inner.shutdown();
                    break;
                }

            };
        }
    }
}
