use crate::work::Work;
use core::{marker::PhantomData, task::Poll};
use futures_core::ready;
use pin_project_lite::pin_project;

pub struct Map<W, T, O> {
    work: W,
    map: T,
    error: PhantomData<fn() -> O>,
}

impl<W: Clone, T: Clone, O> Clone for Map<W, T, O> {
    fn clone(&self) -> Self {
        Map {
            work: self.work.clone(),
            map: self.map.clone(),
            error: PhantomData,
        }
    }
}

impl<W: Copy, T: Copy, O> Copy for Map<W, T, O> {}

unsafe impl<W: Send, T: Send, O> Send for Map<W, T, O> {}

unsafe impl<W: Sync, T: Sync, O> Sync for Map<W, T, O> {}

impl<W, T, O> Map<W, T, O> {
    pub fn new(work: W, map: T) -> Map<W, T, O> {
        Map {
            work,
            map,
            error: PhantomData,
        }
    }
}

impl<W, T, O, C, I> Work<C, I> for Map<W, T, O>
where
    W: Work<C, I>,
    T: Fn(W::Output) -> O,
{
    type Output = O;

    type Error = W::Error;

    type Future<'a>
        = MapFuture<'a, W, C, I, T, O>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: I) -> Self::Future<'a> {
        MapFuture {
            future: self.work.call(context, req),
            map: &self.map,
            error: PhantomData,
        }
    }
}

pin_project! {
  pub struct MapFuture<'a, W: 'a, C: 'a, I, T, O>
  where
    W: Work<C, I>
   {
    #[pin]
    future: W::Future<'a>,
    map: &'a T,
    error: PhantomData<O>
  }
}

impl<'a, W, C, I, T, O> Future for MapFuture<'a, W, C, I, T, O>
where
    W: Work<C, I> + 'a,
    T: Fn(W::Output) -> O,
{
    type Output = Result<O, W::Error>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match ready!(this.future.poll(cx)) {
            Ok(ret) => Poll::Ready(Ok((this.map)(ret))),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
