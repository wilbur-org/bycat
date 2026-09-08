use alloc::{boxed::Box, rc::Rc};

use crate::{Executor, LocalBoxFuture, LocalSpawner, Spawner};

/// A reference-counted executor for local (non-sendable) futures.
///
/// `LocalExecutor` wraps any [`Executor`] that accepts [`LocalBoxFuture`]s and
/// clones via `Rc`, making it suitable for single-threaded contexts.
pub struct LocalExecutor<'js, R = ()> {
    pub(super) inner: Rc<dyn Executor<LocalBoxFuture<'js, R>> + 'js>,
}

impl<'js> LocalSpawner<'js> for LocalExecutor<'js> {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'js,
    {
        self.inner.spawn(Box::pin(work));
    }
}

impl<'js> Spawner<'js> for LocalExecutor<'js> {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'js,
    {
        self.inner.spawn(Box::pin(work));
    }
}

impl<'js, R> Clone for LocalExecutor<'js, R> {
    fn clone(&self) -> Self {
        LocalExecutor {
            inner: self.inner.clone(),
        }
    }
}

impl<'js, R> LocalExecutor<'js, R> {
    /// Creates a new `LocalExecutor` from an [`Executor`] implementation.
    pub fn new<T>(inner: T) -> Self
    where
        T: Executor<LocalBoxFuture<'js, R>> + 'js,
    {
        LocalExecutor {
            inner: Rc::from(inner),
        }
    }

    /// Spawns a future onto the executor.
    pub fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = R> + 'js,
    {
        self.inner.spawn(Box::pin(work));
    }
}

impl<'js, T, R> Executor<T> for LocalExecutor<'js, R>
where
    T: core::future::Future<Output = R> + 'js,
{
    fn spawn(&self, work: T) {
        self.inner.spawn(Box::pin(work));
    }
}

#[cfg(feature = "hyper")]
impl<'js, T, R> hyper::rt::Executor<T> for LocalExecutor<'js, R>
where
    T: core::future::Future<Output = R> + 'js,
{
    fn execute(&self, work: T) {
        self.inner.spawn(Box::pin(work));
    }
}
