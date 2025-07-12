use std::{
    marker::PhantomData,
    mem::transmute,
    sync::Arc,
    task::{Poll, ready},
};

use bycat::Work;
use bycat_error::Error;
use pin_project::pin_project;

pub type Bytes = Vec<u8>;

pub trait CacheStore {
    type GetFuture<'a>: Future<Output = Result<Bytes, Error>>
    where
        Self: 'a;

    type SetFuture<'a>: Future<Output = Result<(), Error>>
    where
        Self: 'a;

    fn get<'a>(&'a self, key: &'a [u8]) -> Self::GetFuture<'a>;
    fn set<'a>(&'a self, key: &'a [u8], value: &'a [u8]) -> Self::SetFuture<'a>;
}

pub trait CacheKey {
    fn key(&self) -> Vec<u8>;
}

pub trait Cached: Sized {
    type IntoFuture<'a>: Future<Output = Result<Bytes, Error>>
    where
        Self: 'a;

    type FromFuture<'a>: Future<Output = Result<Self, Error>>;

    fn into_cached<'a>(&'a mut self) -> Self::IntoFuture<'a>;
    fn from_cached<'a>(bytes: &'a Bytes) -> Self::FromFuture<'a>;
}

pub struct Cache<T> {
    store: T,
}

impl<T> Cache<T> where T: CacheStore {}

pub struct CacheWork<T, W> {
    cache: T,
    work: W,
}

impl<T, W, C, I> Work<C, I> for CacheWork<T, W>
where
    T: CacheStore,
    W: Work<C, I>,
    W::Output: Cached,
    for<'a> W::Output: 'a,
    I: CacheKey,
{
    type Output = W::Output;

    type Error = Error;

    type Future<'a>
        = CacheWorkFuture<'a, T, W, C, I>
    where
        Self: 'a,
        C: 'a,
        W::Output: 'a;

    fn call<'a>(&'a self, context: &'a C, req: I) -> Self::Future<'a> {
        let key = req.key();

        CacheWorkFuture {
            state: CacheState::Start,
            key,
            cache: &self.cache,
            work: &self.work,
            context,
        }
    }
}

#[pin_project(project = GetStateProj)]
enum GetState<'a, T: 'a, O>
where
    T: CacheStore,
    O: Cached,
{
    Start,
    CheckCache {
        #[pin]
        future: T::GetFuture<'a>,
    },
    Decode {
        #[pin]
        future: O::FromFuture<'a>,
    },
}

#[pin_project]
struct GetFuture<'a, T: 'a, O>
where
    T: CacheStore,
    O: Cached,
{
    #[pin]
    state: GetState<'a, T, O>,
    key: Arc<Vec<u8>>,
    value: Option<Vec<u8>>,
    cache: &'a T,
}

impl<'a, T: 'a, O> Future for GetFuture<'a, T, O>
where
    T: CacheStore,
    O: Cached,
{
    type Output = Result<O, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                GetStateProj::Start => {
                    let future = this.cache.get(&this.key);
                    this.state.set(GetState::CheckCache {
                        future: unsafe { transmute::<_, T::GetFuture<'a>>(future) },
                    });
                }
                GetStateProj::CheckCache { future } => {
                    let ret = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    *this.value = Some(ret);

                    let future = O::from_cached(this.value.as_ref().unwrap());

                    this.state.set(GetState::Decode {
                        future: unsafe { transmute(future) },
                    });
                }
                GetStateProj::Decode { future } => return future.poll(cx),
            }
        }
    }
}

#[pin_project(project = SetStateProj)]
enum SetState<'a, T: 'a, W: 'a, C: 'a, I>
where
    T: CacheStore,
    W: Work<C, I>,
    W::Output: Cached + 'a,
{
    Work {
        #[pin]
        future: W::Future<'a>,
    },
    Encode {
        #[pin]
        future: <W::Output as Cached>::IntoFuture<'a>,
    },
    Cache {
        #[pin]
        future: T::SetFuture<'a>,
    },
}

#[pin_project]
struct SetFuture<'a, T: 'a, W: 'a, C: 'a, I>
where
    T: CacheStore,
    W: Work<C, I>,
    W::Output: Cached + 'a,
{
    #[pin]
    state: SetState<'a, T, W, C, I>,
    key: Arc<Vec<u8>>,
}

impl<'a, T: 'a, W: 'a, C: 'a, I> Future for SetFuture<'a, T, W, C, I>
where
    T: CacheStore,
    W: Work<C, I>,
    W::Output: Cached + 'a,
{
    type Output = Result<W::Output, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                SetStateProj::Work { future } => todo!(),
                SetStateProj::Encode { future } => todo!(),
                SetStateProj::Cache { future } => todo!(),
            }
        }
    }
}

#[pin_project]
enum CacheState<'a, T, W, C, I>
where
    T: CacheStore,
    W: Work<C, I>,
    W::Output: Cached,
{
    Start,
    Get(#[pin] GetFuture<'a, T, W::Output>),
    Set(#[pin] SetFuture<'a, T, W, C, I>),
}

#[pin_project]
pub struct CacheWorkFuture<'a, T, W, C, I>
where
    T: CacheStore,
    W: Work<C, I>,
    W::Output: Cached,
{
    state: CacheState<'a, T, W, C, I>,
    cache: &'a T,
    key: Vec<u8>,
    work: &'a W,
    context: &'a C,
}

impl<'a, T, W, C, I> Future for CacheWorkFuture<'a, T, W, C, I>
where
    T: CacheStore,
    W: Work<C, I>,
    W::Output: Cached,
{
    type Output = Result<W::Output, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}
