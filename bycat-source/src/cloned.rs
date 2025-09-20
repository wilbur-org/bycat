use core::{
    future::Future,
    task::{ready, Poll},
};

use crate::{Source, Work};
use alloc::boxed::Box;
use async_stream::try_stream;
use futures::{future::TryJoin, Stream, TryFuture, TryStreamExt};
use heather::{HBoxStream, HSend, HSendSync};
use pin_project_lite::pin_project;

#[derive(Debug, Clone, Copy)]
pub struct AsyncCloned<S, T1, T2> {
    source: S,
    work1: T1,
    work2: T2,
}

impl<S, T1, T2> AsyncCloned<S, T1, T2> {
    pub fn new(source: S, work1: T1, work2: T2) -> AsyncCloned<S, T1, T2> {
        AsyncCloned {
            source,
            work1,
            work2,
        }
    }
}

// impl<S, T1, T2, C> Source<C> for AsyncCloned<S, T1, T2>
// where
//     S: Source<C> + HSend,
//     for<'a> S: 'a,
//     S::Error: HSend,
//     for<'a> S::Stream<'a>: HSend,
//     S::Item: Clone + HSend,
//     T1::Output: HSend,
//     T1: Work<C, S::Item> + Clone + HSend,
//     for<'a> T1: 'a,
//     T1::Error: Into<S::Error> + HSend,
//     for<'a> T1::Future<'a>: HSend,
//     T2: Work<C, S::Item, Output = T1::Output> + Clone + HSend,
//     for<'a> T2: 'a,
//     T2::Error: Into<S::Error> + HSend,
//     for<'a> T2::Future<'a>: HSend,
//     for<'a> C: HSendSync + 'a,
// {
//     type Item = T1::Output;
//     type Error = S::Error;
//     type Stream<'a>
//         = HBoxStream<'a, Result<Self::Item, Self::Error>>
//     where
//         S: 'a,
//         C: 'a,
//         T1: 'a,
//         T2: 'a;

//     fn create_stream<'a>(self, ctx: &'a C) -> Self::Stream<'a> {
//         Box::pin(try_stream! {
//             let stream = self.source.create_stream(ctx);
//             futures::pin_mut!(stream);

//             while let Some(item) = stream.try_next().await? {

//                 yield self.work1.call(ctx, item.clone()).await?;
//                 yield self.work2.call(ctx, item).await?;

//             }
//         })
//     }
// }

impl<S, T1, T2, C> Source<C> for AsyncCloned<S, T1, T2>
where
    S: Source<C>,
    S::Item: Clone,
    T1: Work<C, S::Item>,
    T1::Error: Into<S::Error>,
    T2: Work<C, S::Item, Output = T1::Output, Error = T1::Error>,
{
    type Item = T1::Output;
    type Error = S::Error;
    type Stream<'a>
        = AsyncCloneFuture<'a, C, S, T1, T2>
    where
        S: 'a,
        C: 'a,
        T1: 'a,
        T2: 'a;

    fn create_stream<'a>(self, ctx: &'a C) -> Self::Stream<'a> {
        AsyncCloneFuture {
            source: self.source.create_stream(ctx),
            work1: self.work1,
            work2: self.work2,
            state: CloneState::Next,
            context: ctx,
        }
    }
}

pin_project! {

#[project = CloneProj]
enum CloneState<T1: TryFuture, T2: TryFuture<Error = T1::Error>> {
    Next,
    Work { #[pin] future: TryJoin<T1, T2> },
}

}

pin_project! {
    pub struct AsyncCloneFuture<'a, C: 'a, S: 'a, T1: 'a, T2: 'a>
where
    S: Source<C>,
    T1: Work<C, S::Item>,
    T1::Error: Into<S::Error>,
    T2: Work<C, S::Item, Output = T1::Output, Error = T1::Error>,
{
    #[pin]
    source: S::Stream<'a>,
    work1: T1,
    work2: T2,
    #[pin]
    state: CloneState<T1::Future<'a>, T2::Future<'a>>,
    context: &'a C,
}

}

impl<'a, C: 'a, S: 'a, T1: 'a, T2: 'a> Stream for AsyncCloneFuture<'a, C, S, T1, T2>
where
    S: Source<C>,
    S::Item: Clone,
    T1: Work<C, S::Item>,
    T1::Error: Into<S::Error>,
    T2: Work<C, S::Item, Output = T1::Output, Error = T1::Error>,
{
    type Item = Result<T1::Output, S::Error>;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();
            match this.state.as_mut().project() {
                CloneProj::Next => match ready!(this.source.poll_next(cx)) {
                    Some(Ok(ret)) => {
                        let future1 = this.work1.call(*this.context, ret.clone());
                        let future2 = this.work2.call(*this.context, ret);
                        this.state.set(CloneState::Work {
                            future: unsafe {
                                core::mem::transmute(futures::future::try_join(future1, future2))
                            },
                        });
                    }
                    Some(Err(err)) => return Poll::Ready(Some(Err(err))),
                    None => return Poll::Ready(None),
                },
                CloneProj::Work { future } => {
                    //
                    match ready!(future.poll(cx)) {
                        Ok(ret) => {}
                        Err(err) => {}
                    }
                }
            }
        }
    }
}
