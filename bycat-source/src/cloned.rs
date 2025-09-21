use core::{
    future::Future,
    task::{ready, Poll},
};

use crate::{Source, Work};
use futures::{Stream, TryFuture};

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
    Work { #[pin] future: PairStream<T1, T2> },
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
                                core::mem::transmute(PairStream {
                                    future1: PairState::Working { future: future1 },
                                    future2: PairState::Working { future: future2 },
                                })
                            },
                        });
                    }
                    Some(Err(err)) => return Poll::Ready(Some(Err(err))),
                    None => return Poll::Ready(None),
                },
                CloneProj::Work { future } => match ready!(future.poll_next(cx)) {
                    Some(ret) => return Poll::Ready(Some(ret.map_err(Into::into))),
                    None => {
                        this.state.set(CloneState::Next);
                    }
                },
            }
        }
    }
}

pin_project! {
#[project = PairProj]
enum PairState<T1> {
    Working { #[pin] future: T1 },
    Done,
}
}

pin_project! {
struct PairStream<T1, T2> {
    #[pin]
    future1: PairState<T1>,
    #[pin]
    future2: PairState<T2>,
}

}

impl<T1, T2> Stream for PairStream<T1, T2>
where
    T1: Future,
    T2: Future<Output = T1::Output>,
{
    type Item = T1::Output;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();

            match (
                this.future1.as_mut().project(),
                this.future2.as_mut().project(),
            ) {
                (PairProj::Done, PairProj::Done) => return Poll::Ready(None),
                (PairProj::Done, PairProj::Working { future }) => {
                    let ret = ready!(future.poll(cx));
                    this.future2.set(PairState::Done);
                    return Poll::Ready(Some(ret));
                }
                (PairProj::Working { future }, PairProj::Done) => {
                    let ret = ready!(future.poll(cx));
                    this.future1.set(PairState::Done);
                    return Poll::Ready(Some(ret));
                }
                (PairProj::Working { future: future1 }, PairProj::Working { future: future2 }) => {
                    match future1.poll(cx) {
                        Poll::Pending => {}
                        Poll::Ready(ret) => {
                            this.future1.set(PairState::Done);
                            return Poll::Ready(Some(ret));
                        }
                    }

                    let ret = ready!(future2.poll(cx));
                    this.future2.set(PairState::Done);
                    return Poll::Ready(Some(ret));
                }
            }
        }
    }
}
