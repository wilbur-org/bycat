use alloc::{boxed::Box, rc::Rc, sync::Arc};
use core::any::Any;

use crate::{BlockingSpawner, BoxFuture, LocalBoxFuture, LocalSpawner, Spawner};

/// Object-safe version of [`Spawner`].
///
/// Accepts a boxed, sendable, unit-output future instead of a generic future,
/// which makes it possible to use as a trait object (`dyn DynSpawner`).
pub trait DynSpawner<'a> {
    /// Spawns a boxed sendable future on the executor.
    fn spawn_boxed(&self, work: BoxFuture<'a, ()>);
}

/// Wraps any [`Spawner`] as a [`DynSpawner`].
pub struct BoxedSpawner<'a> {
    inner: Rc<dyn DynSpawner<'a> + 'a>,
}

impl<'a> Clone for BoxedSpawner<'a> {
    fn clone(&self) -> Self {
        BoxedSpawner {
            inner: self.inner.clone(),
        }
    }
}

impl<'a> BoxedSpawner<'a> {
    /// Creates a new `BoxedSpawner` from an existing spawner.
    pub fn new<T>(inner: T) -> Self
    where
        T: Spawner<'a> + 'a,
    {
        BoxedSpawner {
            inner: Rc::new(WrapSpawner(inner)),
        }
    }
}

impl<'a> Spawner<'a> for BoxedSpawner<'a> {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'a,
    {
        self.inner.spawn_boxed(Box::pin(work));
    }
}

impl<'a> DynSpawner<'a> for BoxedSpawner<'a> {
    fn spawn_boxed(&self, work: BoxFuture<'a, ()>) {
        self.inner.spawn_boxed(work);
    }
}

struct WrapSpawner<T>(T);

impl<'a, T> DynSpawner<'a> for WrapSpawner<T>
where
    T: Spawner<'a>,
{
    fn spawn_boxed(&self, work: BoxFuture<'a, ()>) {
        self.0.spawn(work);
    }
}

/// Object-safe version of [`LocalSpawner`].
///
/// Accepts a boxed, local, unit-output future instead of a generic future,
/// which makes it possible to use as a trait object (`dyn DynLocalSpawner`).
pub trait DynLocalSpawner<'a> {
    /// Spawns a boxed local future on the executor.
    fn spawn_boxed(&self, work: LocalBoxFuture<'a, ()>);
}

/// Wraps any [`LocalSpawner`] as a [`DynLocalSpawner`].
pub struct BoxedLocalSpawner<'a> {
    inner: Rc<dyn DynLocalSpawner<'a> + 'a>,
}

impl<'a> Clone for BoxedLocalSpawner<'a> {
    fn clone(&self) -> Self {
        BoxedLocalSpawner {
            inner: self.inner.clone(),
        }
    }
}

impl<'a> BoxedLocalSpawner<'a> {
    /// Creates a new `BoxedLocalSpawner` from an existing local spawner.
    pub fn new<T>(inner: T) -> Self
    where
        T: LocalSpawner<'a> + 'a,
    {
        BoxedLocalSpawner {
            inner: Rc::new(WrapLocalSpawner(inner)),
        }
    }
}

impl<'a> LocalSpawner<'a> for BoxedLocalSpawner<'a> {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'a,
    {
        self.inner.spawn_boxed(Box::pin(work));
    }
}

impl<'a> DynLocalSpawner<'a> for BoxedLocalSpawner<'a> {
    fn spawn_boxed(&self, work: LocalBoxFuture<'a, ()>) {
        self.inner.spawn_boxed(work);
    }
}

struct WrapLocalSpawner<T>(T);

impl<'a, T> DynLocalSpawner<'a> for WrapLocalSpawner<T>
where
    T: LocalSpawner<'a>,
{
    fn spawn_boxed(&self, work: LocalBoxFuture<'a, ()>) {
        self.0.spawn(work);
    }
}

/// A type-erased result returned by [`DynBlockingSpawner`].
#[derive(Debug)]
pub struct DynResult(pub Box<dyn Any + Send + 'static>);

/// Object-safe version of [`BlockingSpawner`].
///
/// Erases the concrete result type by returning the value as a [`DynResult`].
/// The error type is erased to `()`.
pub trait DynBlockingSpawner {
    /// Spawns a boxed blocking closure and returns a boxed future resolving to
    /// a type-erased result.
    fn spawn_blocking_boxed(
        &self,
        work: Box<dyn FnOnce() -> DynResult + Send + 'static>,
    ) -> BoxFuture<'static, Result<DynResult, ()>>;
}

/// Wraps any [`BlockingSpawner`] as a [`DynBlockingSpawner`].
pub struct BoxedBlockingSpawner {
    inner: Arc<dyn DynBlockingSpawner + Send + Sync>,
}

impl Clone for BoxedBlockingSpawner {
    fn clone(&self) -> Self {
        BoxedBlockingSpawner {
            inner: self.inner.clone(),
        }
    }
}

impl BoxedBlockingSpawner {
    /// Creates a new `BoxedBlockingSpawner` from an existing blocking spawner.
    pub fn new<T>(inner: T) -> Self
    where
        T: BlockingSpawner + Send + Sync + 'static,
        T::Error: core::fmt::Debug,
        T::Future<DynResult>: Send,
    {
        BoxedBlockingSpawner {
            inner: Arc::new(WrapBlockingSpawner(inner)),
        }
    }
}

impl BlockingSpawner for BoxedBlockingSpawner {
    type Error = ();
    type Future<R> = BoxFuture<'static, Result<R, ()>>;

    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        let handle = self
            .inner
            .spawn_blocking_boxed(Box::new(move || DynResult(Box::new(work()))));
        Box::pin(async move { handle.await?.0.downcast::<R>().map(|v| *v).map_err(|_| ()) })
    }
}

impl DynBlockingSpawner for BoxedBlockingSpawner {
    fn spawn_blocking_boxed(
        &self,
        work: Box<dyn FnOnce() -> DynResult + Send + 'static>,
    ) -> BoxFuture<'static, Result<DynResult, ()>> {
        self.inner.spawn_blocking_boxed(work)
    }
}

struct WrapBlockingSpawner<T>(T);

impl<T> DynBlockingSpawner for WrapBlockingSpawner<T>
where
    T: BlockingSpawner + Send + Sync + 'static,
    T::Error: core::fmt::Debug,
    T::Future<DynResult>: Send,
{
    fn spawn_blocking_boxed(
        &self,
        work: Box<dyn FnOnce() -> DynResult + Send + 'static>,
    ) -> BoxFuture<'static, Result<DynResult, ()>> {
        let handle: T::Future<DynResult> = self.0.spawn_blocking(work);
        Box::pin(async move { handle.await.map_err(|_| ()) })
    }
}
