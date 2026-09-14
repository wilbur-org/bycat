use bycat_futures::IntoResult;
use bycat_service::{Matcher, Service};
use core::task::{Poll, ready};
use http::{Request, Response};
use pin_project_lite::pin_project;

use crate::{Error, IntoResponse};

pub trait FilteredService<C, B>: Service<C, Request<B>> {
    type CanHandleFuture<'a>: Future<Output = bool>
    where
        Self: 'a,
        C: 'a;

    fn can_handle<'this: 'lifetime, 'ctx: 'lifetime, 'lifetime>(
        &'this self,
        ctx: &'ctx C,
        req: &Request<B>,
    ) -> Self::CanHandleFuture<'lifetime>;
}

#[derive(Debug, Clone, Copy)]
pub struct FilterService<T, M> {
    inner: T,
    filter: M,
}

impl<T, M> FilterService<T, M> {
    pub fn new(inner: T, filter: M) -> Self {
        Self { inner, filter }
    }
}

impl<T, M, C, B> Service<C, Request<B>> for FilterService<T, M>
where
    T: Service<C, Request<B>>,
    M: Matcher<Request<B>>,
{
    type Output = T::Output;
    type Error = T::Error;

    type Future<'a>
        = T::Future<'a>
    where
        Self: 'a,
        C: 'a;

    fn call<'this: 'lifetime, 'ctx: 'lifetime, 'lifetime>(
        &'this self,
        ctx: &'ctx C,
        req: Request<B>,
    ) -> Self::Future<'lifetime> {
        self.inner.call(ctx, req)
    }
}

impl<T, M, C, B> FilteredService<C, B> for FilterService<T, M>
where
    T: Service<C, Request<B>>,
    M: Matcher<Request<B>>,
{
    type CanHandleFuture<'a>
        = core::future::Ready<bool>
    where
        Self: 'a,
        C: 'a;

    fn can_handle<'this: 'lifetime, 'ctx: 'lifetime, 'lifetime>(
        &'this self,
        _ctx: &'ctx C,
        req: &Request<B>,
    ) -> Self::CanHandleFuture<'lifetime> {
        core::future::ready(self.filter.is_match(req))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Or<T1, T2>(pub T1, pub T2);

impl<T1, T2, C, B> Service<C, Request<B>> for Or<T1, T2>
where
    T1: FilteredService<C, B>,
    T1::Output: IntoResponse<B>,
    T1::Error: Into<Error>,
    T2: FilteredService<C, B>,
    T2::Output: IntoResponse<B>,
    T2::Error: Into<Error>,
{
    type Output = Response<B>;
    type Error = Error;

    type Future<'a>
        = OrFuture<'a, T1, T2, C, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'this: 'lifetime, 'ctx: 'lifetime, 'lifetime>(
        &'this self,
        ctx: &'ctx C,
        req: Request<B>,
    ) -> Self::Future<'lifetime> {
        let future = self.0.can_handle(ctx, &req);

        OrFuture {
            state: OrFutureState::CheckLeft {
                left: &self.0,
                right: &self.1,
                ctx,
                req: Some(req),
                future,
            },
        }
    }
}

pin_project! {
    #[project = OrFutureProj]
    enum OrFutureState<'a, T1: 'a, T2: 'a, C: 'a, B>
    where
        T1: FilteredService<C, B>,
        T2: FilteredService<C, B>,
    {
        CheckLeft {
            left: &'a T1,
            right: &'a T2,
            ctx: &'a C,
            req: Option<Request<B>>,
            #[pin]
            future: T1::CanHandleFuture<'a>,
        },
        CheckRight {
            right: &'a T2,
            ctx: &'a C,
            req: Option<Request<B>>,
            #[pin]
            future: T2::CanHandleFuture<'a>,
        },
        Left {
            #[pin]
            future: T1::Future<'a>,
        },
        Right {
            #[pin]
            future: T2::Future<'a>,
        },
        NotFound
    }
}

pin_project! {
    pub struct OrFuture<'a, T1: 'a, T2: 'a, C: 'a, B>
    where
        T1: FilteredService<C, B>,
        T2: FilteredService<C, B>,
    {
        #[pin]
        state: OrFutureState<'a, T1, T2, C, B>,
    }
}

impl<'a, T1, T2, C, B> Future for OrFuture<'a, T1, T2, C, B>
where
    T1: FilteredService<C, B> + 'a,
    T1::Output: IntoResponse<B>,
    T1::Error: Into<Error>,
    T2: FilteredService<C, B> + 'a,
    T2::Output: IntoResponse<B>,
    T2::Error: Into<Error>,
    C: 'a,
{
    type Output = Result<Response<B>, Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                OrFutureProj::CheckLeft {
                    left,
                    right,
                    ctx,
                    req,
                    future,
                } => {
                    let can_handle = ready!(future.poll(cx));
                    let req = req.take().expect("request missing from OrFuture");

                    if can_handle {
                        let left = *left;
                        let ctx = *ctx;
                        let future = left.call(ctx, req);
                        this.state.set(OrFutureState::Left { future });
                    } else {
                        let right = *right;
                        let ctx = *ctx;
                        let future = right.can_handle(ctx, &req);
                        this.state.set(OrFutureState::CheckRight {
                            right,
                            ctx,
                            req: Some(req),
                            future,
                        });
                    }
                }
                OrFutureProj::CheckRight {
                    right,
                    ctx,
                    req,
                    future,
                } => {
                    let can_handle = ready!(future.poll(cx));
                    let req = req.take().expect("request missing from OrFuture");

                    if can_handle {
                        let right = *right;
                        let ctx = *ctx;
                        let future = right.call(ctx, req);
                        this.state.set(OrFutureState::Right { future });
                    } else {
                        this.state.set(OrFutureState::NotFound);
                    }
                }
                OrFutureProj::Left { future } => match ready!(future.poll(cx)).into_result() {
                    Ok(ret) => return Poll::Ready(Ok(ret.into_response())),
                    Err(err) => {
                        let err: Error = err.into();
                        return Poll::Ready(Err(err));
                    }
                },
                OrFutureProj::Right { future } => match ready!(future.poll(cx)).into_result() {
                    Ok(ret) => return Poll::Ready(Ok(ret.into_response())),
                    Err(err) => {
                        let err: Error = err.into();
                        return Poll::Ready(Err(err));
                    }
                },
                OrFutureProj::NotFound => return Poll::Ready(Err(Error::not_found())),
            }
        }
    }
}
