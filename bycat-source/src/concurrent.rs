use core::task::{ready, Poll};

use bycat::Work;
use futures::{
    stream::{Fuse, FuturesUnordered},
    Stream, StreamExt,
};
use pin_project_lite::pin_project;

use crate::Source;

pub struct Concurrent<S, T> {
    source: S,
    work: T,
}

impl<S, T> Concurrent<S, T> {
    pub fn new(source: S, work: T) -> Concurrent<S, T> {
        Concurrent { source, work }
    }
}

impl<S, T, C> Source<C> for Concurrent<S, T>
where
    S: Source<C>,

    T: Work<C, S::Item>,
    T::Error: Into<S::Error>,
{
    type Error = S::Error;
    type Item = T::Output;

    type Stream<'a>
        = ConcurrentStream<'a, C, S, T>
    where
        S: 'a,
        T: 'a,
        C: 'a;

    fn create_stream<'a>(self, ctx: &'a C) -> Self::Stream<'a> {
        ConcurrentStream {
            stream: self.source.create_stream(ctx).fuse(),
            work: self.work,
            futures: FuturesUnordered::new(),
            context: ctx,
        }
    }
}

pin_project! {
  pub struct ConcurrentStream<'a, C: 'a, S: Source<C>, T: Work<C, S::Item>>
  where
    T: 'a,
    S: 'a
   {
    #[pin]
    stream: Fuse<S::Stream<'a>>,
    work: T,
    #[pin]
    futures: FuturesUnordered<T::Future<'a>>,
    context: &'a C
}
}

impl<'a, C: 'a, S: Source<C> + 'a, T: Work<C, S::Item> + 'a> Stream
    for ConcurrentStream<'a, C, S, T>
where
    T::Error: Into<S::Error>,
{
    type Item = Result<T::Output, S::Error>;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();

            let done = match this.stream.poll_next(cx) {
                Poll::Ready(Some(Ok(ret))) => {
                    let future = this.work.call(&this.context, ret);
                    this.futures
                        .as_mut()
                        .push(unsafe { core::mem::transmute::<_, T::Future<'a>>(future) });
                    false
                }
                Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(err))),
                Poll::Ready(None) => true,
                Poll::Pending => false,
            };

            if done && this.futures.as_mut().is_empty() {
                return Poll::Ready(None);
            }

            let ret = ready!(this.futures.as_mut().poll_next(cx));
            return Poll::Ready(ret.map(|m| m.map_err(Into::into)));
        }
    }
}
