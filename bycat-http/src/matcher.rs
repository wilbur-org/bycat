use alloc::boxed::Box;
use bycat_service::{Matcher, Service};
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Poll, ready},
};
use http::{Request, Response};
use pin_project_lite::pin_project;

use crate::{Error, IntoResponse};

pub trait FilteredService<C, B>: Service<C, Request<B>> {
    type CanHandleFuture<'a>: Future<Output = bool>
    where
        Self: 'a,
        C: 'a,
        B: 'a;

    fn can_handle<'this: 'lifetime, 'ctx: 'lifetime, 'req: 'lifetime, 'lifetime>(
        &'this self,
        ctx: &'ctx C,
        req: &'req Request<B>,
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
        C: 'a,
        B: 'a;

    fn can_handle<'this: 'lifetime, 'ctx: 'lifetime, 'req: 'lifetime, 'lifetime>(
        &'this self,
        _ctx: &'ctx C,
        req: &'req Request<B>,
    ) -> Self::CanHandleFuture<'lifetime> {
        core::future::ready(self.filter.is_match(req))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Or<T1, T2, B>(pub T1, pub T2, PhantomData<fn() -> B>);

impl<T1, T2, B> Or<T1, T2, B> {
    pub fn new(left: T1, right: T2) -> Self {
        Self(left, right, PhantomData)
    }
}

impl<T1, T2, C, B> Service<C, Request<B>> for Or<T1, T2, B>
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
        let req = Box::new(req);
        // SAFETY: `req` is immediately moved into `OrFuture`, and the boxed
        // allocation keeps the request at a stable address while the filter
        // future is stored in `OrFutureState::CheckLeft`.
        let future = self.0.can_handle(ctx, unsafe { request_ref(&req) });

        OrFuture {
            state: OrFutureState::CheckLeft {
                left: &self.0,
                right: &self.1,
                ctx,
                future,
            },
            req: Some(req),
        }
    }
}

unsafe fn request_ref<'a, B>(req: &Request<B>) -> &'a Request<B> {
    // SAFETY: The caller must ensure the referenced request remains alive and
    // is not moved for `'a`, and that any future borrowing it is dropped before
    // the request is moved or dropped.
    unsafe { &*(req as *const Request<B>) }
}

pin_project! {
    #[project = OrFutureProj]
    enum OrFutureState<'a, T1: 'a, T2: 'a, C: 'a, B: 'a>
    where
        T1: FilteredService<C, B>,
        T2: FilteredService<C, B>,
    {
        CheckLeft {
            left: &'a T1,
            right: &'a T2,
            ctx: &'a C,
            #[pin]
            future: T1::CanHandleFuture<'a>,
        },
        CheckRight {
            right: &'a T2,
            ctx: &'a C,
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
        NotFound {
            _body: PhantomData<fn() -> B>,
        }
    }
}

pin_project! {
    pub struct OrFuture<'a, T1: 'a, T2: 'a, C: 'a, B: 'a>
    where
        T1: FilteredService<C, B>,
        T2: FilteredService<C, B>,
    {
        #[pin]
        state: OrFutureState<'a, T1, T2, C, B>,
        req: Option<Box<Request<B>>>,
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
    B: 'a,
{
    type Output = Result<Response<B>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                OrFutureProj::CheckLeft {
                    left,
                    right,
                    ctx,
                    future,
                } => {
                    let can_handle = ready!(future.poll(cx));
                    let left = *left;
                    let right = *right;
                    let ctx = *ctx;

                    this.state
                        .set(OrFutureState::NotFound { _body: PhantomData });

                    if can_handle {
                        let req = *this.req.take().expect("request missing from OrFuture");
                        let future = left.call(ctx, req);
                        this.state.set(OrFutureState::Left { future });
                    } else {
                        let req = this.req.as_deref().expect("request missing from OrFuture");
                        // SAFETY: the request is pinned behind a `Box` stored in
                        // `this.req`; the current filter future was dropped by
                        // setting `state` to `NotFound`, and the right filter
                        // future is stored in the state before polling resumes.
                        let future = right.can_handle(ctx, unsafe { request_ref(req) });
                        this.state
                            .set(OrFutureState::CheckRight { right, ctx, future });
                    }
                }
                OrFutureProj::CheckRight { right, ctx, future } => {
                    let can_handle = ready!(future.poll(cx));
                    let right = *right;
                    let ctx = *ctx;

                    this.state
                        .set(OrFutureState::NotFound { _body: PhantomData });

                    if can_handle {
                        let req = *this.req.take().expect("request missing from OrFuture");
                        let future = right.call(ctx, req);
                        this.state.set(OrFutureState::Right { future });
                    } else {
                        this.req.take();
                        return Poll::Ready(Err(Error::not_found()));
                    }
                }
                OrFutureProj::Left { future } => match ready!(future.poll(cx)) {
                    Ok(ret) => return Poll::Ready(Ok(ret.into_response())),
                    Err(err) => return Poll::Ready(Err(err.into())),
                },
                OrFutureProj::Right { future } => match ready!(future.poll(cx)) {
                    Ok(ret) => return Poll::Ready(Ok(ret.into_response())),
                    Err(err) => return Poll::Ready(Err(err.into())),
                },
                OrFutureProj::NotFound { .. } => {
                    return Poll::Ready(Err(Error::not_found()));
                }
            }
        }
    }
}
