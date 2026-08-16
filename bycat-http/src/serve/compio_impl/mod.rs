mod listener;
mod servable;
use crate::serve::Server;

use self::servable::*;

use ::bycat_service::Service;
use bycat_executor::CompioExecutor;
use bycat_server::Shutdown;
use compio::net::ToSocketAddrsAsync;

#[derive(Debug, Clone, Copy)]
pub struct Compio<T> {
    work: T,
    upgrade: bool,
}

impl<T> Compio<T> {
    pub fn new(work: T) -> Self {
        Self {
            work,
            upgrade: false,
        }
    }

    pub fn upgradable(mut self, upgrade: bool) -> Self {
        self.upgrade = upgrade;
        self
    }

    pub async fn serve<C>(self, ctx: C, addr: impl ToSocketAddrsAsync) -> Result<(), std::io::Error>
    where
        T: Service<
                C,
                http::Request<crate::body::Body>,
                Output = http::Response<crate::body::Body>,
                Error = crate::Error,
            >
            + Clone
            + 'static
            + Send,
        for<'a> T::Future<'a>: Send,
        C: Send + Clone + 'static,
    {
        let server = Server::new(CompioExecutor::default(), CompioServable(self.work, ctx))
            .with_upgrade(self.upgrade);

        let listener = compio::net::TcpListener::bind(addr).await?;

        server.serve(listener, &Shutdown::new()).await;

        Ok(())
    }
}
