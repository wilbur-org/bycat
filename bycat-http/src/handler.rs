use bycat::Work;
use bycat_error::{BoxError, Error};
use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, ready},
};
use http::{Request, Response};
use pin_project_lite::pin_project;

use crate::{FromRequest, IntoResponse};

pub fn handler<B, C, T, I, M>(func: T) -> FuncHandler<T, I, B, C, M>
where
    T: Func<B, C, I>,
    <T::Future as Future>::Output: IntoResponse<B>,
    I: FromRequest<C, B, M>,
{
    FuncHandler {
        func,
        i: PhantomData,
    }
}

pub struct FuncHandler<T, I, B, C, M> {
    func: T,
    i: PhantomData<(B, C, I, M)>,
}

impl<T, I, B, C, M> Clone for FuncHandler<T, I, B, C, M>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        FuncHandler {
            func: self.func.clone(),
            i: PhantomData,
        }
    }
}

unsafe impl<T, I, B, C, M> Send for FuncHandler<T, I, B, C, M> where T: Send {}

unsafe impl<T, I, B, C, M> Sync for FuncHandler<T, I, B, C, M> where T: Sync {}

impl<B, C, T, I, M> Work<C, Request<B>> for FuncHandler<T, I, B, C, M>
where
    T: Func<B, C, I>,
    <T::Future as Future>::Output: IntoResponse<B>,
    <<T::Future as Future>::Output as IntoResponse<B>>::Error: Into<BoxError>,
    I: FromRequest<C, B, M>,
{
    type Output = Response<B>;

    type Error = Error;

    type Future<'a>
        = HandlerFnFuture<'a, I, T, C, B, M>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        HandlerFnFuture {
            state: HandlerFnFutureState::Request {
                future: I::from_request(req, context),
            },
            handler: &self.func,
        }
    }
}

pub trait Func<B, C, I> {
    type Future: Future;

    fn call(&self, req: I) -> Self::Future;
}

impl<B, C, F, U> Func<B, C, ()> for F
where
    F: Fn() -> U,
    U: Future,
{
    type Future = U;

    fn call(&self, _req: ()) -> Self::Future {
        (self)()
    }
}

macro_rules! funcs {
    ($first: ident) => {
        impl<B, C, F, U, $first> Func<B, C, ($first,)> for F
        where
            F: Fn($first) -> U,
            U: Future,
        {
            type Future = U;

            fn call(&self, req: ($first,)) -> Self::Future {
                (self)(req.0)
            }
        }
    };
    ($first: ident, $($rest:ident),+) => {
        funcs!($($rest),+);

        impl<B, C, F, U, $first, $($rest),*> Func<B, C, ($first, $($rest),*)> for F
        where
            F: Fn($first, $($rest),*) -> U,
            U: Future,

        {
            type Future = U;

            #[allow(non_snake_case)]
            fn call(&self, ($first, $($rest),*): ($first,$($rest),*)) -> Self::Future {
                (self)($first, $($rest),*)
            }
        }
    }
}

funcs!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
);

pin_project! {
#[project = HandlerFnFutureStateProj]
enum HandlerFnFutureState<'a, T, H, C: 'a, B, M>
where
    T: FromRequest<C, B, M>,
    H: Func<B, C, T>
{
    Request {
        #[pin]
        future: T::Future<'a>
    },
    Handler {
        #[pin]
        future: H::Future
    },
}
}

pin_project! {

    pub struct HandlerFnFuture<'a, T, H, C: 'a, B, M>
    where
        T: FromRequest<C, B, M>,
        H: Func<B, C, T>

    {
        #[pin]
        state: HandlerFnFutureState<'a, T, H, C, B, M>,
        handler: &'a H
    }


}

impl<'a, T, H, C: 'a, B, M> Future for HandlerFnFuture<'a, T, H, C, B, M>
where
    T: FromRequest<C, B, M>,
    H: Func<B, C, T>,
    <H::Future as Future>::Output: IntoResponse<B>,
    <<H::Future as Future>::Output as IntoResponse<B>>::Error: Into<BoxError>,
{
    type Output = Result<Response<B>, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                HandlerFnFutureStateProj::Request { future } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    let future = this.handler.call(ret);

                    this.state.set(HandlerFnFutureState::Handler { future });
                }
                HandlerFnFutureStateProj::Handler { future } => {
                    let ret = ready!(future.poll(cx));
                    return Poll::Ready(ret.into_response().map_err(Error::new));
                }
            }
        }
    }
}
