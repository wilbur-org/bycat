use bycat_executor::CompioExecutor;
use bycat_service::Service;

use crate::serve::{Conn, Listener, Servable};

pub struct CompioServable<T, C>(pub T, pub C);

impl<T, C, L> Servable<CompioExecutor, L> for CompioServable<T, C>
where
    L: Listener + 'static,
    L::Io: Send,
    L::Addr: Send,
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
    type Future<'a>
        = CompioServerFuture<L, T, C>
    where
        Self: 'a;

    fn call<'a>(&'a self, conn: Conn<L, CompioExecutor>) -> Self::Future<'a> {
        CompioServerFuture::Init {
            work: Some(self.0.clone()),
            conn: Some(conn),
            context: Some(self.1.clone()),
        }
    }
}

pin_project_lite::pin_project! {
    #[project = LocalTokioServerFutureProj]
    pub enum CompioServerFuture<L, T, C>
    where
        L: Listener
    {
       Init {
        work: Option<T>,
        conn: Option<Conn<L, CompioExecutor>>,
        context: Option<C>

       },
       Done
    }
}

impl<L, T, C> Future for CompioServerFuture<L, T, C>
where
    L: Listener + 'static,
    L::Io: Send,
    L::Addr: Send,
    T: Service<
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

                compio::runtime::spawn(async move {
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
                })
                .detach();

                self.set(Self::Done);

                core::task::Poll::Ready(())
            }
            LocalTokioServerFutureProj::Done => panic!("Poll after done"),
        }
    }
}
