/// Spawns a single future or task onto an executor.
///
/// This is the low-level trait used by the higher-level [`LocalExecutor`] and
/// [`SendExecutor`] wrappers. Implementors receive the task directly and are
/// responsible for driving it to completion.
pub trait Executor<T> {
    /// Spawns `work` onto the executor.
    fn spawn(&self, work: T);
}

impl<'a, T, V> Executor<T> for &'a V
where
    V: Executor<T> + ?Sized,
{
    fn spawn(&self, work: T) {
        (**self).spawn(work);
    }
}

/// Spawns sendable futures that produce no value.
///
/// This is useful for contexts that only need to fire-and-forget sendable
/// futures with a unit output, such as background task loops.
pub trait Spawner<'a> {
    /// Spawns `work` on the executor.
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'a;
}

impl<'a, 'b, T> Spawner<'a> for &'b T
where
    T: Spawner<'a> + ?Sized,
{
    fn spawn<U>(&self, work: U)
    where
        U: core::future::Future<Output = ()> + Send + 'a,
    {
        (**self).spawn(work);
    }
}

impl<'a, 'b, T> Spawner<'a> for &'b mut T
where
    T: Spawner<'a> + ?Sized,
{
    fn spawn<U>(&self, work: U)
    where
        U: core::future::Future<Output = ()> + Send + 'a,
    {
        (**self).spawn(work);
    }
}

/// Spawns local (non-sendable) futures that produce no value.
///
/// Like [`Spawner`], but does not require the future to be `Send`, allowing it
/// to be used with single-threaded executors.
pub trait LocalSpawner<'a> {
    /// Spawns `work` on the local executor.
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'a;
}

impl<'a, 'b, T> LocalSpawner<'a> for &'b T
where
    T: LocalSpawner<'a> + ?Sized,
{
    fn spawn<U>(&self, work: U)
    where
        U: core::future::Future<Output = ()> + 'a,
    {
        (**self).spawn(work);
    }
}

impl<'a, 'b, T> LocalSpawner<'a> for &'b mut T
where
    T: LocalSpawner<'a> + ?Sized,
{
    fn spawn<U>(&self, work: U)
    where
        U: core::future::Future<Output = ()> + 'a,
    {
        (**self).spawn(work);
    }
}

/// Spawns blocking work onto a thread pool.
///
/// The returned future resolves to the result of the blocking closure, or an
/// error if the executor failed to run or return it.
pub trait BlockingSpawner {
    /// Future that resolves once the blocking work finishes.
    type Future<R>: core::future::Future<Output = Result<R, Self::Error>>;

    /// Error returned when the blocking task cannot be completed.
    type Error;

    /// Spawns `work` on a blocking thread pool.
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static;
}

/// Provides access to a [`Spawner`].
pub trait HasSpawner<'a> {
    /// The spawner type.
    type Spawner: Spawner<'a>;

    /// Returns a reference to the spawner.
    fn spawner(&self) -> &Self::Spawner;
}

/// Provides access to a [`LocalSpawner`].
pub trait HasLocalSpawner<'a> {
    /// The local spawner type.
    type Spawner: LocalSpawner<'a>;

    /// Returns a reference to the local spawner.
    fn local_spawner(&self) -> &Self::Spawner;
}

/// Provides access to a [`BlockingSpawner`].
pub trait HasBlockingSpawner {
    /// The blocking spawner type.
    type Spawner: BlockingSpawner;

    /// Returns a reference to the blocking spawner.
    fn blocking_spawner(&self) -> &Self::Spawner;
}
