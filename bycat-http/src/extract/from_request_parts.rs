use core::{
    mem::transmute,
    pin::Pin,
    task::{Context, Poll, ready},
};

use bycat_error::Error;
use http::{HeaderMap, Uri, request::Parts};
use pin_project_lite::pin_project;

use crate::router::UrlParams;

pub trait FromRequestParts<C>: Sized {
    type Future<'a>: Future<Output = Result<Self, Error>>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, state: &'a C) -> Self::Future<'a>;
}

impl<C> FromRequestParts<C> for HeaderMap {
    type Future<'a>
        = core::future::Ready<Result<HeaderMap, Error>>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, _state: &'a C) -> Self::Future<'a> {
        core::future::ready(Ok(parts.headers.clone()))
    }
}

impl<C> FromRequestParts<C> for Uri {
    type Future<'a>
        = core::future::Ready<Result<Self, Error>>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, _state: &'a C) -> Self::Future<'a> {
        core::future::ready(Ok(parts.uri.clone()))
    }
}

impl<C> FromRequestParts<C> for UrlParams {
    type Future<'a>
        = core::future::Ready<Result<Self, Error>>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, _state: &'a C) -> Self::Future<'a> {
        core::future::ready(
            parts
                .extensions
                .get::<Self>()
                .cloned()
                .ok_or(Error::new("Missing UrlParams")),
        )
    }
}

// Tuples

pin_project! {
    #[project = FromQuestStateProj]
    enum FromQuestState<'a, T1, T2, C: 'a>
    where
        T1: FromRequestParts<C>,
        T2: FromRequestParts<C>,
    {
        Start,
        First {
            #[pin]
             future: T1::Future<'a>
        },
        Last {
            #[pin]
            future: T2::Future<'a>,
            results: Option<T1>
        }
    }
}

pin_project! {
    struct FromQuestFuture<'a, T1, T2, C: 'a>
    where
        T1: FromRequestParts<C>,
        T2: FromRequestParts<C>,
    {
        #[pin]
        state: FromQuestState<'a, T1, T2, C>,
        ctx: &'a C,
        parts: &'a mut Parts
    }
}

impl<'a, T1, T2, C: 'a> Future for FromQuestFuture<'a, T1, T2, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
{
    type Output = Result<(T1, T2), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                FromQuestStateProj::Start => {
                    let future = T1::from_request_parts(this.parts, this.ctx);
                    this.state.set(FromQuestState::First {
                        future: unsafe { transmute(future) },
                    })
                }
                FromQuestStateProj::First { future } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    let future = T2::from_request_parts(this.parts, this.ctx);

                    this.state.set(FromQuestState::Last {
                        future: unsafe { transmute(future) },
                        results: Some(ret),
                    });
                }
                FromQuestStateProj::Last { future, results } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    let ret1 = results.take().unwrap();

                    return Poll::Ready(Ok((ret1, ret)));
                }
            }
        }
    }
}

impl<T1, C> FromRequestParts<C> for (T1,)
where
    T1: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts1<'a, T1, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, state: &'a C) -> Self::Future<'a> {
        FromRequestParts1 {
            future: T1::from_request_parts(parts, state),
        }
    }
}

impl<T1, T2, C> FromRequestParts<C> for (T1, T2)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts2<'a, T1, T2, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, state: &'a C) -> Self::Future<'a> {
        FromRequestParts2 {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx: state,
                parts,
            },
        }
    }
}

impl<T1, T2, T3, C> FromRequestParts<C> for (T1, T2, T3)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts3<'a, T1, T2, T3, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts3 {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

impl<T1, T2, T3, T4, C> FromRequestParts<C> for (T1, T2, T3, T4)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts4<'a, T1, T2, T3, T4, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts4 {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

impl<T1, T2, T3, T4, T5, C> FromRequestParts<C> for (T1, T2, T3, T4, T5)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequest5Future<'a, T1, T2, T3, T4, T5, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequest5Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, C> FromRequestParts<C> for (T1, T2, T3, T4, T5, T6)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequest6Future<'a, T1, T2, T3, T4, T5, T6, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequest6Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

// 1
pin_project! {
    pub struct FromRequestParts1<'a, T1, C: 'a>
where
    T1: FromRequestParts<C>,
{
   #[pin]
    future: T1::Future<'a>
}

}

impl<'a, T1, C: 'a> Future for FromRequestParts1<'a, T1, C>
where
    T1: FromRequestParts<C>,
{
    type Output = Result<(T1,), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();
        match ready!(this.future.poll(cx)) {
            Ok(ret) => Poll::Ready(Ok((ret,))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

//

pin_project! {
    pub struct FromRequestParts2<'a,  T1, T2, C: 'a>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
{
    #[pin]
    state: FromQuestFuture<'a, T1, T2, C>,
}

}

impl<'a, T1, T2, C> Future for FromRequestParts2<'a, T1, T2, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
{
    type Output = Result<(T1, T2), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok((ret1, ret2)) => Poll::Ready(Ok((ret1, ret2))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

// 3

pin_project! {
    pub struct FromRequestParts3<'a,  T1, T2, T3, C: 'a>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
{

    #[pin]
    state: FromQuestFuture<'a, (T1, T2), T3, C>,
}

}

impl<'a, T1, T2, T3, C> Future for FromRequestParts3<'a, T1, T2, T3, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
{
    type Output = Result<(T1, T2, T3), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2), ret3)) => Poll::Ready(Ok((ret1, ret2, ret3))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

// 4

pin_project! {
    pub struct FromRequestParts4<'a,  T1, T2, T3, T4, C: 'a>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
{
    #[pin]
    state: FromQuestFuture<'a, (T1, T2, T3), T4, C>,
}

}

impl<'a, T1, T2, T3, T4, C> Future for FromRequestParts4<'a, T1, T2, T3, T4, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
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
    pub struct FromRequest5Future<'a, T1, T2, T3, T4, T5, C: 'a>
    where
        T1: FromRequestParts<C>,
        T2: FromRequestParts<C>,
        T3: FromRequestParts<C>,
        T4: FromRequestParts<C>,
        T5: FromRequestParts<C>,
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4), T5, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, C: 'a> Future for FromRequest5Future<'a, T1, T2, T3, T4, T5, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
{
    type Output = Result<(T1, T2, T3, T4, T5), Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.project().state.poll(cx)) {
            Ok(((ret1, ret2, ret3, ret4), ret5)) => Poll::Ready(Ok((ret1, ret2, ret3, ret4, ret5))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

pin_project! {
    pub struct FromRequest6Future<'a, T1, T2, T3, T4, T5, T6, C: 'a>
    where
        T1: FromRequestParts<C>,
        T2: FromRequestParts<C>,
        T3: FromRequestParts<C>,
        T4: FromRequestParts<C>,
        T5: FromRequestParts<C>,
        T6: FromRequestParts<C>,
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5), T6, C>,

    }
}

impl<'a, T1, T2, T3, T4, T5, T6, C: 'a> Future for FromRequest6Future<'a, T1, T2, T3, T4, T5, T6, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
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

impl<T1, T2, T3, T4, T5, T6, T7, C> FromRequestParts<C> for (T1, T2, T3, T4, T5, T6, T7)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts7Future<'a, T1, T2, T3, T4, T5, T6, T7, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts7Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts7Future<'a, T1, T2, T3, T4, T5, T6, T7, C: 'a>
    where
        T1: FromRequestParts<C>,
        T2: FromRequestParts<C>,
        T3: FromRequestParts<C>,
        T4: FromRequestParts<C>,
        T5: FromRequestParts<C>,
        T6: FromRequestParts<C>,
        T7: FromRequestParts<C>,
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6), T7, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, C: 'a> Future
    for FromRequestParts7Future<'a, T1, T2, T3, T4, T5, T6, T7, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, C> FromRequestParts<C> for (T1, T2, T3, T4, T5, T6, T7, T8)
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts8Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts8Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts8Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, C: 'a>
    where
        T1: FromRequestParts<C>,
        T2: FromRequestParts<C>,
        T3: FromRequestParts<C>,
        T4: FromRequestParts<C>,
        T5: FromRequestParts<C>,
        T6: FromRequestParts<C>,
        T7: FromRequestParts<C>,
        T8: FromRequestParts<C>,
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7), T8, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, C: 'a> Future
    for FromRequestParts8Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, C>
where
    T1: FromRequestParts<C>,
    T2: FromRequestParts<C>,
    T3: FromRequestParts<C>,
    T4: FromRequestParts<C>,
    T5: FromRequestParts<C>,
    T6: FromRequestParts<C>,
    T7: FromRequestParts<C>,
    T8: FromRequestParts<C>,
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, C> FromRequestParts<C>
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
    T9: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts9Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts9Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts9Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, C: 'a>
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
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8), T9, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, C: 'a> Future
    for FromRequestParts9Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, C>
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C> FromRequestParts<C>
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
    T10: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts10Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts10Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts10Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C: 'a>
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
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9), T10, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C: 'a> Future
    for FromRequestParts10Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, C>
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

// Repeat the above pattern for FromRequestParts11 to FromRequestParts16

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C> FromRequestParts<C>
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
    T11: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts11Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts11Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts11Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C: 'a>
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
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10), T11, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C: 'a> Future
    for FromRequestParts11Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, C>
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C> FromRequestParts<C>
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
    T12: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts12Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts12Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts12Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C: 'a>
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
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11), T12, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C: 'a> Future
    for FromRequestParts12Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, C>
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

// Repeat the above pattern for FromRequestParts13 to FromRequestParts16

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C> FromRequestParts<C>
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
    T13: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts13Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts13Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts13Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C: 'a>
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
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12), T13, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C: 'a> Future
    for FromRequestParts13Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, C>
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C> FromRequestParts<C>
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
    T14: FromRequestParts<C>,
{
    type Future<'a>
        =
        FromRequestParts14Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C>
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts14Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts14Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C: 'a>
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
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13), T14, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C: 'a> Future
    for FromRequestParts14Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, C>
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, C> FromRequestParts<C>
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
    T15: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts15Future<
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
    >
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts15Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts15Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, C: 'a>
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
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14), T15, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, C: 'a> Future
    for FromRequestParts15Future<
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

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, C> FromRequestParts<C>
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
    T16: FromRequestParts<C>,
{
    type Future<'a>
        = FromRequestParts16Future<
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
    >
    where
        C: 'a;

    fn from_request_parts<'a>(parts: &'a mut Parts, ctx: &'a C) -> Self::Future<'a> {
        FromRequestParts16Future {
            state: FromQuestFuture {
                state: FromQuestState::Start,
                ctx,
                parts,
            },
        }
    }
}

pin_project! {
    pub struct FromRequestParts16Future<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, C: 'a>
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
        T16: FromRequestParts<C>,
    {
        #[pin]
        state: FromQuestFuture<'a, (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15), T16, C>,
    }
}

impl<'a, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, C: 'a> Future
    for FromRequestParts16Future<
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
    T16: FromRequestParts<C>,
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
