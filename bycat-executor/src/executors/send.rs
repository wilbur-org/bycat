use alloc::{boxed::Box, sync::Arc};

use crate::{BoxFuture, Executor, Spawner};

/// A thread-safe, reference-counted executor for sendable futures.
///
/// `SendExecutor` wraps any [`Executor`] that accepts [`BoxFuture`]s and clones
/// via `Arc`, allowing it to be shared across threads.
pub struct SendExecutor<'js, R = ()> {
    pub(super) inner: Arc<dyn Executor<BoxFuture<'js, R>> + Send + Sync + 'js>,
}

impl<'js, R> Clone for SendExecutor<'js, R> {
    fn clone(&self) -> Self {
        SendExecutor {
            inner: self.inner.clone(),
        }
    }
}

impl<'js> Spawner<'js> for SendExecutor<'js> {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'js,
    {
        self.inner.spawn(Box::pin(work));
    }
}

impl<'js, R> SendExecutor<'js, R> {
    /// Creates a new `SendExecutor` from a thread-safe [`Executor`] implementation.
    pub fn new<T>(inner: T) -> Self
    where
        T: Executor<BoxFuture<'js, R>> + 'js + Send + Sync,
    {
        SendExecutor {
            inner: Arc::from(inner),
        }
    }

    /// Spawns a sendable future onto the executor.
    pub fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = R> + Send + 'js,
    {
        self.inner.spawn(Box::pin(work));
    }
}

impl<'js, T, R> Executor<T> for SendExecutor<'js, R>
where
    T: core::future::Future<Output = R> + Send + 'js,
{
    fn spawn(&self, work: T) {
        self.inner.spawn(Box::pin(work));
    }
}

#[cfg(feature = "hyper")]
impl<'js, T, R> hyper::rt::Executor<T> for SendExecutor<'js, R>
where
    T: core::future::Future<Output = R> + Send + 'js,
{
    fn execute(&self, work: T) {
        self.inner.spawn(Box::pin(work));
    }
}
