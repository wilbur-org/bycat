use std::pin::pin;

use hyper::server::conn::http1::Builder;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use super::{Connection, listener::Listener, shutdown::Shutdown};

pub trait Socket: hyper::rt::Read + hyper::rt::Write {}

impl<T> Socket for T where T: hyper::rt::Read + hyper::rt::Write {}

pub trait Servable<E> {
    type Future<'a, S, A>: Future<Output = ()>
    where
        Self: 'a,
        S: Socket + Unpin + Send + 'static,
        A: Send + 'static;

    fn call<S, A>(&self, conn: Connection<S, A, E>) -> Self::Future<'_, S, A>
    where
        S: Socket + Unpin + Send + 'static,
        A: Send + 'static;
}

pub struct Server<T, E> {
    service: T,
    builder: Builder,
    executor: E,
}

impl<T, E> Server<T, E> {
    pub fn new(executor: E, service: T) -> Server<T, E> {
        Server {
            service,
            builder: Builder::new(),
            executor,
        }
    }
}

impl<T, E> Server<T, E>
where
    T: Servable<E>,
    E: Clone,
{
    pub async fn listen<F>(
        self,
        addr: impl tokio::net::ToSocketAddrs,
        kill: F,
    ) -> Result<(), tokio::io::Error>
    where
        F: Future<Output = ()>,
    {
        let listener = TcpListener::bind(addr).await?;

        self.serve(listener, kill).await;

        Ok(())
    }

    pub async fn serve<L, F>(self, mut listener: L, kill: F)
    where
        L: Listener,
        F: Future<Output = ()>,
    {
        let mut kill = pin!(kill);
        let shutdown = Shutdown::new();

        loop {
            tokio::select! {
                biased;
                (stream, address) = listener.accept() => {
                    let stream = TokioIo::new(stream);

                    let conn = Connection::new(self.executor.clone(),self.builder.clone(), shutdown.clone(), stream, address);

                    self.service.call(conn).await;
                }
                _ = &mut kill => {
                    shutdown.shutdown().await;
                    break;
                }

            };
        }
    }
}
