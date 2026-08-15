pub use crate::serve::{listener::*, server::*};
use ::bycat_service::Work;
use bycat_executor::{LocalTokioExecutor, TokioExecutor};

#[derive(Debug, Clone)]
pub struct TokioServer<T, C>(pub T, pub C);

impl<T, C, L> Servable<TokioExecutor, L> for TokioServer<T, C>
where
    L: Listener + 'static,
    L::Io: Send,
    L::Addr: Send,
    T: Work<
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
    type Future<'a>
        = TokioServerFuture<L, T, C>
    where
        Self: 'a;

    fn call(&self, conn: Conn<L, TokioExecutor>) -> Self::Future<'_> {
        TokioServerFuture::Init {
            work: Some(self.0.clone()),
            conn: Some(conn),
            context: Some(self.1.clone()),
        }
    }
}

pin_project_lite::pin_project! {
    #[project = TokioServerFutureProj]
    pub enum TokioServerFuture<L, T, C>
    where
        L: Listener
    {
       Init {
        work: Option<T>,
        conn: Option<Conn<L, TokioExecutor>>,
        context: Option<C>

       },
       Done
    }
}

impl<L, T, C> Future for TokioServerFuture<L, T, C>
where
    L: Listener + 'static,
    L::Io: Send,
    L::Addr: Send,
    T: Work<
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
                            let req = req.map(Into::into);

                            match work.call(&context, req).await {
                                Ok(ret) => Ok(ret),
                                Err(err) => {
                                    alloc::println!("Error {}", err);
                                    Err(err)
                                }
                            }
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

impl<T, C, L> Servable<LocalTokioExecutor, L> for TokioServer<T, C>
where
    L: Listener + 'static,
    L::Io: Send,
    L::Addr: Send,
    T: Work<
            C,
            http::Request<crate::body::Body>,
            Output = http::Response<crate::body::Body>,
            Error = crate::Error,
        > + Clone
        + 'static,
    C: Clone + 'static,
{
    type Future<'a>
        = LocalTokioServerFuture<L, T, C>
    where
        Self: 'a;

    fn call(&self, conn: Conn<L, LocalTokioExecutor>) -> Self::Future<'_> {
        LocalTokioServerFuture::Init {
            work: Some(self.0.clone()),
            conn: Some(conn),
            context: Some(self.1.clone()),
        }
    }
}

pin_project_lite::pin_project! {
    #[project = LocalTokioServerFutureProj]
    pub enum LocalTokioServerFuture<L, T, C>
    where
        L: Listener
    {
       Init {
        work: Option<T>,
        conn: Option<Conn<L, LocalTokioExecutor>>,
        context: Option<C>

       },
       Done
    }
}

impl<L, T, C> Future for LocalTokioServerFuture<L, T, C>
where
    L: Listener + 'static,
    L::Io: Send,
    L::Addr: Send,
    T: Work<
            C,
            http::Request<crate::body::Body>,
            Output = http::Response<crate::body::Body>,
            Error = crate::Error,
        > + Clone
        + 'static,
    C: Clone + 'static,
{
    type Output = ();

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.as_mut().project();

        match this {
            LocalTokioServerFutureProj::Init {
                conn,
                work,
                context,
            } => {
                let work = work.take().unwrap();
                let conn = conn.take().unwrap();
                let context = context.take().unwrap();

                tokio::task::spawn_local(async move {
                    let svc = hyper::service::service_fn(move |req| {
                        let work = work.clone();
                        let context = context.clone();
                        async move {
                            let req = req.map(Into::into);
                            match work.call(&context, req).await {
                                Ok(ret) => Ok(ret),
                                Err(err) => {
                                    alloc::println!("Error {}", err);
                                    Err(err)
                                }
                            }
                        }
                    });

                    if let Err(err) = conn.serve_connection(svc).await {
                        alloc::eprintln!("server error: {}", err);
                    }
                });

                self.set(Self::Done);

                core::task::Poll::Ready(())
            }
            LocalTokioServerFutureProj::Done => panic!("Poll after done"),
        }
    }
}
