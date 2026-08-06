use alloc::{marker::PhantomData, task::Poll};
use bycat::{Matcher, Work};
use bycat_futures::IntoResult;
use futures::ready;
use http::{Request, Response};
use pin_project_lite::pin_project;

use crate::{Error, IntoResponse};

pub trait FilteredWork<C, B>: Work<C, Request<B>> {
    fn can_handle(&self, ctx: &C, req: &Request<B>) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct FilterWork<T, M> {
    inner: T,
    filter: M,
}

impl<T, M> FilterWork<T, M> {
    pub fn new(inner: T, filter: M) -> Self {
        Self { inner, filter }
    }
}

impl<T, M, C, B> Work<C, Request<B>> for FilterWork<T, M>
where
    T: Work<C, Request<B>>,
    M: Matcher<Request<B>>,
{
    type Output = T::Output;
    type Error = T::Error;

    type Future<'a>
        = T::Future<'a>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, ctx: &'a C, req: Request<B>) -> Self::Future<'a> {
        self.inner.call(ctx, req)
    }
}

impl<T, M, C, B> FilteredWork<C, B> for FilterWork<T, M>
where
    T: Work<C, Request<B>>,
    M: Matcher<Request<B>>,
{
    fn can_handle(&self, _ctx: &C, req: &Request<B>) -> bool {
        self.filter.is_match(req)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Or<T1, T2>(pub T1, pub T2);

impl<T1, T2, C, B> Work<C, Request<B>> for Or<T1, T2>
where
    T1: FilteredWork<C, B>,
    T1::Output: IntoResponse<B>,
    T1::Error: Into<Error>,
    T2: FilteredWork<C, B>,
    T2::Output: IntoResponse<B>,
    T2::Error: Into<Error>,
{
    type Output = Response<B>;
    type Error = Error;

    type Future<'a>
        = OrFuture<T1::Future<'a>, T2::Future<'a>, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, ctx: &'a C, req: Request<B>) -> Self::Future<'a> {
        let state = if self.0.can_handle(ctx, &req) {
            OrFutureState::Left {
                future: self.0.call(ctx, req),
            }
        } else if self.1.can_handle(ctx, &req) {
            OrFutureState::Right {
                future: self.1.call(ctx, req),
            }
        } else {
            OrFutureState::NotFound
        };

        OrFuture {
            state: state,
            body: PhantomData,
        }
    }
}

pin_project! {
    #[project = OrFutureProj]
    enum OrFutureState<T1, T2> {
        Left {
            #[pin]
            future: T1,
        },
        Right {
            #[pin]
            future: T2,
        },
        NotFound
    }
}

pin_project! {
    pub struct OrFuture<T1, T2, B> {
        #[pin]
        state: OrFutureState<T1, T2>,
        body: PhantomData<B>
    }
}

impl<T1, T2, B> Future for OrFuture<T1, T2, B>
where
    T1: Future,
    T1::Output: IntoResult,
    <T1::Output as IntoResult>::Output: IntoResponse<B>,
    <T1::Output as IntoResult>::Error: Into<Error>,
    T2: Future,
    T2::Output: IntoResult,
    <T2::Output as IntoResult>::Output: IntoResponse<B>,
    <T2::Output as IntoResult>::Error: Into<Error>,
{
    type Output = Result<Response<B>, Error>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match this.state.project() {
            OrFutureProj::Left { future } => match ready!(future.poll(cx)).into_result() {
                Ok(ret) => Poll::Ready(Ok(ret.into_response())),
                Err(err) => {
                    let err: Error = err.into();
                    Poll::Ready(Err(err))
                }
            },
            OrFutureProj::Right { future } => match ready!(future.poll(cx)).into_result() {
                Ok(ret) => Poll::Ready(Ok(ret.into_response())),
                Err(err) => {
                    let err: Error = err.into();
                    Poll::Ready(Err(err))
                }
            },
            OrFutureProj::NotFound => Poll::Ready(Err(Error::not_found())),
        }
    }
}
