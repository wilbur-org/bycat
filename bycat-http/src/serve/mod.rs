mod connection;
#[cfg(feature = "smol")]
mod futures;
mod listener;
mod server;

pub use self::{connection::Connection, listener::*, server::*};

pub use bycat_service::Shutdown;
pub use hyper::rt::Executor;

#[cfg(feature = "serve-tokio")]
use crate::body::Body;
#[cfg(feature = "serve-tokio")]
use ::{bycat::Work, bycat_error::Error, http_body_util::BodyExt};

#[cfg(feature = "serve-tokio")]
pub async fn serve<T, C, A>(addr: A, context: C, service: T) -> Result<(), tokio::io::Error>
where
    A: tokio::net::ToSocketAddrs,
    T: Work<
            C,
            http::Request<crate::body::Body>,
            Output = http::Response<crate::body::Body>,
            Error = Error,
        >
        + Clone
        + 'static
        + Send,
    for<'a> T::Future<'a>: Send,
    C: Send + Clone + 'static,
{
    let server = Server::new(TokioExecutor::default(), TokioServer(service, context));

    let listener = tokio::net::TcpListener::bind(addr).await?;

    server.serve(listener, &Shutdown::new()).await;

    Ok(())
}

#[cfg(feature = "serve-tokio")]
struct TokioServer<T, C>(T, C);

#[cfg(feature = "serve-tokio")]
impl<T, C, L> Servable<TokioExecutor, L> for TokioServer<T, C>
where
    L: Listener + 'static,
    L::Io: Send,
    L::Addr: Send,
    T: Work<
            C,
            http::Request<crate::body::Body>,
            Output = http::Response<crate::body::Body>,
            Error = Error,
        >
        + Clone
        + 'static
        + Send,
    for<'a> T::Future<'a>: Send,
    C: Send + Clone + 'static,
{
    type Future<'a>
        = TokioServerFuture<L, T, C>
    where
        Self: 'a;

    fn call(&self, conn: Connection<L, TokioExecutor>) -> Self::Future<'_> {
        TokioServerFuture::Init {
            work: Some(self.0.clone()),
            conn: Some(conn),
            context: Some(self.1.clone()),
        }
    }
}

#[cfg(feature = "serve-tokio")]
pin_project_lite::pin_project! {
    #[project = TokioServerFutureProj]
    pub enum TokioServerFuture<L, T, C>
    where
        L: Listener
    {
       Init {
        work: Option<T>,
        conn: Option<Connection<L, TokioExecutor>>,
        context: Option<C>

       },
       Done
    }
}
#[cfg(feature = "serve-tokio")]
impl<L, T, C> Future for TokioServerFuture<L, T, C>
where
    L: Listener + 'static,
    L::Io: Send,
    L::Addr: Send,
    T: Work<
            C,
            http::Request<crate::body::Body>,
            Output = http::Response<crate::body::Body>,
            Error = Error,
        >
        + Clone
        + 'static
        + Send,
    for<'a> T::Future<'a>: Send,
    C: Send + Clone + 'static,
{
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.as_mut().project();

        match this {
            TokioServerFutureProj::Init {
                conn,
                work,
                context,
            } => {
                let work = work.take().unwrap();
                let conn = conn.take().unwrap();
                let context = context.take().unwrap();

                tokio::spawn(async move {
                    let svc = hyper::service::service_fn(move |req| {
                        let work = work.clone();
                        let context = context.clone();
                        async move {
                            let req = req.map(|body: hyper::body::Incoming| {
                                Body::from_streaming(body.map_err(Error::new))
                            });

                            work.call(&context, req).await
                        }
                    });

                    if let Err(err) = conn.serve_connection(svc).await {
                        alloc::eprintln!("server error: {}", err);
                    }
                });

                self.set(Self::Done);

                core::task::Poll::Ready(())
            }
            TokioServerFutureProj::Done => panic!("Poll after done"),
        }
    }
}
