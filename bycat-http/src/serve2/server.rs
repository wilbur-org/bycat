use alloc::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use bycat::Work;
use bycat_service::Shutdown;
use futures::FutureExt;
use http::Request;
use hyper::{body::Incoming, server::conn::http1::Builder};
use pin_project_lite::pin_project;

use crate::{Error, IntoResponse, error::BoxError, serve::Listener, serve2::conn::HyperConn};

pin_project! {
    pub struct Server<L, E, W, C, B>

    {
        listener: L,
        executor: E,
        builder: Builder,
        work: W,
        context: C,
        shutdown: Shutdown,
        body: PhantomData<B>
    }
}

impl<L, E, W, C, B> Server<L, E, W, C, B>
where
    L: Listener,
    L::Io: 'static,
    W: Work<C, Request<Incoming>> + Clone,
    W::Error: Into<Error>,
    W::Output: IntoResponse<B>,
    C: Clone,
    B: http_body::Body + 'static,
    B::Error: Into<BoxError>,
{
    pub fn new(
        listener: L,
        executor: E,
        builder: Builder,
        work: W,
        context: C,
        shutdown: Shutdown,
    ) -> Self {
        Server {
            listener,
            executor,
            builder,
            work,
            context,
            shutdown,
            body: PhantomData,
        }
    }

    pub async fn serve(self) {
        let Server {
            mut listener,
            executor,
            builder,
            work,
            context,
            shutdown,
            ..
        } = self;

        let mut wait = shutdown.wait().fuse();

        loop {
            futures::select_biased! {
                next = listener.accept().fuse() => {
                    let (socket, addr) = next;
                    let context = context.clone();
                    let work = work.clone();

                    let svc = hyper::service::service_fn(move |req| {
                        let work = work.clone();
                        let context = context.clone();
                        async move {
                            match work.call(&context, req).await {
                                Ok(ret) => Ok(ret.into_response()),
                                Err(err) => {
                                    //
                                    Err(err.into())
                                }
                            }
                        }
                    });

                    let conn = builder.serve_connection(socket, svc);
                    shutdown.watch(HyperConn { conn }).await;
                }
                _ = &mut wait => {
                    break;
                }
            };
        }
    }
}

pub struct HyperService<W, C, B> {
    work: W,
    context: C,
    body: PhantomData<B>,
}

impl<W, C, B> HyperService<W, C, B> {
    pub fn new(work: W, context: C) -> Self {
        Self {
            work,
            context,
            body: PhantomData,
        }
    }
}

impl<W, C, B> hyper::service::Service<Request<Incoming>> for HyperService<W, C, B>
where
    W: Work<C, Request<Incoming>> + Clone,
    W::Error: Into<Error>,
    W::Output: IntoResponse<B>,
    B: http_body::Body + 'static,
    B::Error: Into<BoxError>,
    C: Clone,
{
    type Response = hyper::Response<B>;
    type Error = BoxError;
    type Future = HyperServiceFuture<'static, W, C, B>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        HyperServiceFuture {
            future: self.work.call(&self.context, req),
            body: PhantomData,
        }
    }
}

pin_project! {
    pub struct HyperServiceFuture<'a, W: 'a, C: 'a, B>
    where
        W: Work<C, Request<Incoming>>,
        W::Error: Into<Error>,
        W::Output: IntoResponse<B>,
    {
        #[pin]
        future: W::Future<'a>,
        body: PhantomData<B>,
    }
}

impl<'a, W, C, B> Future for HyperServiceFuture<'a, W, C, B>
where
    W: Work<C, Request<Incoming>> + 'a,
    W::Error: Into<Error>,
    W::Output: IntoResponse<B>,
    B: http_body::Body + 'static,
    B::Error: Into<BoxError>,
{
    type Output = Result<hyper::Response<B>, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.future.poll(cx) {
            Poll::Ready(Ok(output)) => Poll::Ready(Ok(output.into_response())),
            Poll::Ready(Err(err)) => Poll::Ready(Err(err.into())),
            Poll::Pending => Poll::Pending,
        }
    }
}

// #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
// pub struct StaticWork<T>(pub T);

// pin_project! {
//     #[project = StaticWorkProj]
//     enum StaticWorkState<'a, C:'a, R, T: 'a> where T: Work<C, R> {
//         Init {
//             context: Option<C>,
//             request: Option<R>,
//         },
//         Future {
//             #[pin]
//             future: T::Future<'a>,
//         },
//         Done,
//     }
// }

// pub struct StaticWorkFuture<'a, C, R, T>
// where
//     T: Work<C, R>,
// {
//     state: StaticWorkState<'a, C, R, T>,
//     work: T,
// }

// impl<C, R, T> Future for StaticWorkFuture<C, R, T>
// where
//     T: Work<C, R>,
// {
//     type Output = Result<T::Output, T::Error>;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         let this = self.project();
//         match this.state.project() {
//             StaticWorkProj::Init { context, request } => {
//                 let context = context.take().expect("context is None");
//                 let request = request.take().expect("request is None");
//                 let future = this.work.call(&context, request);
//                 *this.state = StaticWorkState::Future { future };
//                 self.poll(cx)
//             }
//             StaticWorkProj::Future { future } => future.poll(cx),
//             StaticWorkProj::Done => panic!("poll called after completion"),
//         }
//     }
// }
