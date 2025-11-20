use bycat_service::Shutdown;
use futures::FutureExt;
use hyper::server::conn::http1::Builder;

use super::{Connection, listener::Listener};

pub trait Servable<E, L>
where
    L: Listener,
{
    type Future<'a>: Future<Output = ()>
    where
        Self: 'a;

    fn call(&self, conn: Connection<L, E>) -> Self::Future<'_>;
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

                    self.service.call(conn).await;
                }
                _ = &mut wait => {
                    inner.shutdown();
                    break;
                }

            };
        }
    }
}
