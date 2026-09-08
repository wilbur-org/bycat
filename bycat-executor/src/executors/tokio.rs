use crate::{BlockingSpawner, Executor, LocalSpawner, Spawner};

/// Executor adapter that spawns tasks on the Tokio runtime.
///
/// Requires an active Tokio runtime. Futures are spawned with
/// [`tokio::task::spawn`] and are therefore `Send` and `'static`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokioExecutor;

impl<T> Executor<T> for TokioExecutor
where
    T: core::future::Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn spawn(&self, work: T) {
        tokio::task::spawn(work);
    }
}

impl Spawner<'static> for TokioExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'static,
    {
        tokio::task::spawn(work);
    }
}

impl BlockingSpawner for TokioExecutor {
    type Future<R> = tokio::task::JoinHandle<R>;
    type Error = tokio::task::JoinError;
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        tokio::task::spawn_blocking(work)
    }
}

#[cfg(feature = "hyper")]
impl<T> hyper::rt::Executor<T> for TokioExecutor
where
    T: core::future::Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn execute(&self, work: T) {
        tokio::task::spawn(work);
    }
}

/// Local executor adapter that spawns tasks on the current Tokio local set.
///
/// Futures are spawned with [`tokio::task::spawn_local`] and do not need to be
/// `Send`, but they must be `'static`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LocalTokioExecutor;

impl<T> Executor<T> for LocalTokioExecutor
where
    T: core::future::Future + 'static,
    T::Output: 'static,
{
    fn spawn(&self, work: T) {
        tokio::task::spawn_local(work);
    }
}

impl Spawner<'static> for LocalTokioExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        tokio::task::spawn_local(work);
    }
}

impl LocalSpawner<'static> for LocalTokioExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        tokio::task::spawn_local(work);
    }
}

impl BlockingSpawner for LocalTokioExecutor {
    type Error = tokio::task::JoinError;
    type Future<R> = tokio::task::JoinHandle<R>;
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        tokio::task::spawn_blocking(work)
    }
}

#[cfg(feature = "hyper")]
impl<T> hyper::rt::Executor<T> for LocalTokioExecutor
where
    T: core::future::Future + 'static,
    T::Output: 'static,
{
    fn execute(&self, work: T) {
        tokio::task::spawn_local(work);
    }
}
