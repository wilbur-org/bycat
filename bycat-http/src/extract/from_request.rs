use super::from_request_parts::FromRequestParts;
#[cfg(feature = "std")]
use crate::body::Body;
use bycat_error::Error;
use core::{
    mem::transmute,
    pin::Pin,
    task::{Context, Poll, ready},
};
use http::{Request, request::Parts};
use pin_project_lite::pin_project;

mod internal {
    pub struct ViaReq;
    pub struct ViaParts;
}

pub trait FromRequest<C, B, M = internal::ViaReq>: Sized {
    type Future<'a>: Future<Output = Result<Self, Error>>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a>;
}

impl<C, B> FromRequest<C, B> for Request<B> {
    type Future<'a>
        = core::future::Ready<Result<Request<B>, Error>>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, _state: &'a C) -> Self::Future<'a> {
        core::future::ready(Ok(parts))
    }
}

impl<T, C, B> FromRequest<C, B, internal::ViaParts> for T
where
    T: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestPartsFuture<'a, T, C>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        let (parts, _) = parts.into_parts();
        FromRequestPartsFuture {
            ctx: state,
            parts,
            state: ViaPartsState::Start,
        }
    }
}

pin_project! {
    #[project = ViaPartsStateProj]
    enum ViaPartsState<'a, T, C: 'a>
    where
        T: FromRequestParts<C>
    {
        Start,
        Parts {
            #[pin]
            future: T::Future<'a>
        }
    }
}

pin_project! {
   pub struct FromRequestPartsFuture<'a, T, C>
where
    T: FromRequestParts<C>,
{
    ctx: &'a C,
    parts: Parts,
    #[pin]
    state: ViaPartsState<'a, T, C>,
}

}

impl<'a, T, C> Future for FromRequestPartsFuture<'a, T, C>
where
    T: FromRequestParts<C>,
{
    type Output = Result<T, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                ViaPartsStateProj::Start => {
                    let future = T::from_request_parts(this.parts, this.ctx);
                    this.state.set(ViaPartsState::Parts {
                        future: unsafe { transmute(future) },
                    });
                }
                ViaPartsStateProj::Parts { future } => return future.poll(cx),
            }
        }
    }
}

#[cfg(feature = "std")]
impl<C> FromRequest<C, Body> for Body {
    type Future<'a>
        = core::future::Ready<Result<Self, Error>>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<Body>, _state: &'a C) -> Self::Future<'a> {
        core::future::ready(Ok(parts.into_body()))
    }
}

impl<C, B> FromRequest<C, B> for () {
    type Future<'a>
        = core::future::Ready<Result<(), Error>>
    where
        C: 'a;

    fn from_request<'a>(_parts: Request<B>, _state: &'a C) -> Self::Future<'a> {
        core::future::ready(Ok(()))
    }
}

pin_project! {
#[project = FromRequestStateProj]
enum FromRequestState<'a, B, T1, T2, C: 'a>
where
    T1: FromRequestParts<C>,
    T2: FromRequest<C, B>,
{
    Start,
    Parts {
        #[pin]
        future: T1::Future<'a>
    },
    Request {
        #[pin]
        future: T2::Future<'a>,
        ret: Option<T1>
    },
}
}

pin_project! {

struct FromRequestFuture<'a, T1, T2, B, C: 'a>
where
    T1: FromRequestParts<C>,
    T2: FromRequest<C, B>,
{
    parts: Option<Parts>,
    body: Option<B>,
    ctx: &'a C,
    #[pin]
    state: FromRequestState<'a, B, T1, T2, C>,
}



}

impl<'a, T1, T2, B, C: 'a> FromRequestFuture<'a, T1, T2, B, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequest<C, B>,
{
    pub fn new(ctx: &'a C, req: Request<B>) -> FromRequestFuture<'a, T1, T2, B, C> {
        let (parts, body) = req.into_parts();

        FromRequestFuture {
            parts: Some(parts),
            body: Some(body),
            ctx,
            state: FromRequestState::Start,
        }
    }
}

impl<'a, T1, T2, B, C> Future for FromRequestFuture<'a, T1, T2, B, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequest<C, B>,
{
    type Output = Result<(T1, T2), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                FromRequestStateProj::Start => {
                    let future = T1::from_request_parts(this.parts.as_mut().unwrap(), this.ctx);
                    // SAFETY: We're transmuting the future lifetime
                    //          This should be safe, because the the parts and context is owned by this struct
                    //          And only accessed through the future.
                    this.state.set(FromRequestState::Parts {
                        future: unsafe { transmute(future) },
                    });
                }
                FromRequestStateProj::Parts { future } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    let future = T2::from_request(
                        Request::from_parts(this.parts.take().unwrap(), this.body.take().unwrap()),
                        this.ctx,
                    );

                    //
                    // SAFETY: We're transmuting the future's lifetime
                    //         This should be safe, because the context belongs to this struct
                    this.state.set(FromRequestState::Request {
                        future: unsafe { transmute(future) },
                        ret: Some(ret),
                    });
                }
                FromRequestStateProj::Request { future, ret } => {
                    let ret2 = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    return Poll::Ready(Ok((ret.take().unwrap(), ret2)));
                }
            }
        }
    }
}

pin_project! {
    pub struct FromRequest1Future<'a, T, C: 'a, B>
    where
        T: FromRequest<C, B>
    {
        #[pin]
        future: T::Future<'a>
    }
}

impl<'a, T, C: 'a, B> Future for FromRequest1Future<'a, T, C, B>
where
    T: FromRequest<C, B>,
{
    type Output = Result<(T,), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let ret = ready!(self.project().future.poll(cx));
        Poll::Ready(ret.map(|m| (m,)))
    }
}

pin_project! {

pub struct FromRequest2Future<'a, T1, T2,C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, T1, T2, B, C>,
}

}

impl<'a, T1, T2, C: 'a, B> Future for FromRequest2Future<'a, T1, T2, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequest<C, B>,
{
    type Output = Result<(T1, T2), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok((ret1, ret2)) => Poll::Ready(Ok((ret1, ret2))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

pin_project! {

pub struct FromRequest3Future<'a, T1, T2, T3, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2), T3, B, C>,
}

}

impl<'a, T1, T2, T3, C: 'a, B> Future for FromRequest3Future<'a, T1, T2, T3, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2), ret3)) => Poll::Ready(Ok((ret1, ret2, ret3))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

pin_project! {

pub struct FromRequest4Future<'a, T1, T2, T3, T4, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3), T4, B, C>,
}

}

impl<'a, T1, T2, T3, T4, C: 'a, B> Future for FromRequest4Future<'a, T1, T2, T3, T4, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3), ret4)) => Poll::Ready(Ok((ret1, ret2, ret3, ret4))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

pin_project! {

pub struct FromRequest5Future<'a, T1, T2, T3, T4, T5, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4), T5, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, C: 'a, B> Future for FromRequest5Future<'a, T1, T2, T3, T4, T5, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4), ret5)) => Poll::Ready(Ok((ret1, ret2, ret3, ret4, ret5))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

// 6

pin_project! {

pub struct FromRequest6Future<'a, T1, T2, T3, T4, T5, T6, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5), T6, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, C: 'a, B> Future
    for FromRequest6Future<'a, T1, T2, T3, T4, T5, T6, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4, ret5), ret6)) => {
                Poll::Ready(Ok((ret1, ret2, ret3, ret4, ret5, ret6)))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

// Tuples of FromRequest

impl<T1, C, B> FromRequest<C, B> for (T1,)
where
    T1: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest1Future<'a, T1, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest1Future {
            future: T1::from_request(parts, state),
        }
    }
}

impl<T1, T2, C, B> FromRequest<C, B> for (T1, T2)
where
    T1: FromRequestParts<C>,
    T2: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest2Future<'a, T1, T2, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest2Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

impl<T1, T2, T3, C, B> FromRequest<C, B> for (T1, T2, T3)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest3Future<'a, T1, T2, T3, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest3Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

impl<T1, T2, T3, T4, C, B> FromRequest<C, B> for (T1, T2, T3, T4)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest4Future<'a, T1, T2, T3, T4, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest4Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

impl<T1, T2, T3, T4, T5, C, B> FromRequest<C, B> for (T1, T2, T3, T4, T5)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest5Future<'a, T1, T2, T3, T4, T5, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest5Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, C, B> FromRequest<C, B> for (T1, T2, T3, T4, T5, T6)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest6Future<'a, T1, T2, T3, T4, T5, T6, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest6Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 7

pin_project! {

pub struct FromRequest7Future<'a, T1, T2, T3, T4, T5, T6, T7, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6), T7, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, C: 'a, B> Future
    for FromRequest7Future<'a, T1, T2, T3, T4, T5, T6, T7, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6, T7), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4, ret5, ret6), ret7)) => {
                Poll::Ready(Ok((ret1, ret2, ret3, ret4, ret5, ret6, ret7)))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, C, B> FromRequest<C, B> for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest7Future<'a, T1, T2, T3, T4, T5, T6, T7, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest7Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 8

pin_project! {

pub struct FromRequest8Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7), T8, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, C: 'a, B> Future
    for FromRequest8Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6, T7, T8), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4, ret5, ret6, ret7), ret8)) => {
                Poll::Ready(Ok((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8)))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, C, B> FromRequest<C, B> for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest8Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest8Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 9

pin_project! {

pub struct FromRequest9Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8), T9, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, C: 'a, B> Future
    for FromRequest9Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6, T7, T8, T9), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8), ret9)) => {
                Poll::Ready(Ok((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9)))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, C, B> FromRequest<C, B>
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest9Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest9Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 10

pin_project! {

pub struct FromRequest10Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9), T10, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C: 'a, B> Future
    for FromRequest10Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9), ret10)) => Poll::Ready(Ok(
                (ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10),
            )),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C, B> FromRequest<C, B>
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest10Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest10Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 11

pin_project! {

pub struct FromRequest11Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10), T11, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C: 'a, B> Future
    for FromRequest11Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10), ret11)) => {
                Poll::Ready(Ok((
                    ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11,
                )))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C, B> FromRequest<C, B>
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest11Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest11Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 12

pin_project! {

pub struct FromRequest12Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11), T12, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C: 'a, B> Future
    for FromRequest12Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11), ret12)) => {
                Poll::Ready(Ok((
                    ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11, ret12,
                )))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C, B> FromRequest<C, B>
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest12Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest12Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 13

pin_project! {

pub struct FromRequest13Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12), T13, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C: 'a, B> Future
    for FromRequest13Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok((
                (ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11, ret12),
                ret13,
            )) => Poll::Ready(Ok((
                ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11, ret12, ret13,
            ))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C, B> FromRequest<C, B>
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest13Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest13Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 14

pin_project! {

pub struct FromRequest14Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13), T14, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C: 'a, B> Future
    for FromRequest14Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequest<C, B>,
{
    type Output = Result<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok((
                (ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11, ret12, ret13),
                ret14,
            )) => Poll::Ready(Ok((
                ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11, ret12, ret13,
                ret14,
            ))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C, B> FromRequest<C, B>
    for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest14Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C, B>
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest14Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 15

pin_project! {

pub struct FromRequest15Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequestParts<C>,
    T15: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14), T15, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, C: 'a, B> Future
    for FromRequest15Future<
        'a,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
        T9,
        T10,
        T11,
        T12,
        T13,
        T14,
        T15,
        C,
        B,
    >
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequestParts<C>,
    T15: FromRequest<C, B>,
{
    type Output = Result<
        (
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
            T10,
            T11,
            T12,
            T13,
            T14,
            T15,
        ),
        Error,
    >;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok((
                (
                    ret1,
                    ret2,
                    ret3,
                    ret4,
                    ret5,
                    ret6,
                    ret7,
                    ret8,
                    ret9,
                    ret10,
                    ret11,
                    ret12,
                    ret13,
                    ret14,
                ),
                ret15,
            )) => Poll::Ready(Ok((
                ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11, ret12, ret13,
                ret14, ret15,
            ))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, C, B> FromRequest<C, B>
    for (
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
        T9,
        T10,
        T11,
        T12,
        T13,
        T14,
        T15,
    )
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequestParts<C>,
    T15: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest15Future<
        'a,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
        T9,
        T10,
        T11,
        T12,
        T13,
        T14,
        T15,
        C,
        B,
    >
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest15Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}

// 16

pin_project! {

pub struct FromRequest16Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, C: 'a, B>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequestParts<C>,
    T15: FromRequestParts<C>,
    T16: FromRequest<C, B>,
{
    #[pin]
    state: FromRequestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15), T16, B, C>,
}

}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, C: 'a, B> Future
    for FromRequest16Future<
        'a,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
        T9,
        T10,
        T11,
        T12,
        T13,
        T14,
        T15,
        T16,
        C,
        B,
    >
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequestParts<C>,
    T15: FromRequestParts<C>,
    T16: FromRequest<C, B>,
{
    type Output = Result<
        (
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
            T10,
            T11,
            T12,
            T13,
            T14,
            T15,
            T16,
        ),
        Error,
    >;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok((
                (
                    ret1,
                    ret2,
                    ret3,
                    ret4,
                    ret5,
                    ret6,
                    ret7,
                    ret8,
                    ret9,
                    ret10,
                    ret11,
                    ret12,
                    ret13,
                    ret14,
                    ret15,
                ),
                ret16,
            )) => Poll::Ready(Ok((
                ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11, ret12, ret13,
                ret14, ret15, ret16,
            ))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, C, B> FromRequest<C, B>
    for (
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
        T9,
        T10,
        T11,
        T12,
        T13,
        T14,
        T15,
        T16,
    )
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
    T9: FromRequestParts<C>,
    T10: FromRequestParts<C>,
    T11: FromRequestParts<C>,
    T12: FromRequestParts<C>,
    T13: FromRequestParts<C>,
    T14: FromRequestParts<C>,
    T15: FromRequestParts<C>,
    T16: FromRequest<C, B>,
{
    type Future<'a>
        = FromRequest16Future<
        'a,
        T1,
        T2,
        T3,
        T4,
        T5,
        T6,
        T7,
        T8,
        T9,
        T10,
        T11,
        T12,
        T13,
        T14,
        T15,
        T16,
        C,
        B,
    >
    where
        C: 'a;

    fn from_request<'a>(parts: Request<B>, state: &'a C) -> Self::Future<'a> {
        FromRequest16Future {
            state: FromRequestFuture::new(state, parts),
        }
    }
}
