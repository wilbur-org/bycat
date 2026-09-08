#![no_std]

use core::pin::Pin;

use alloc::{boxed::Box, rc::Rc, sync::Arc};

pub type BoxFuture<'a, T> = Pin<Box<dyn core::future::Future<Output = T> + Send + 'a>>;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn core::future::Future<Output = T> + 'a>>;

// Executor trait and executor implementations for local and sendable futures.

extern crate alloc;

pub trait Executor<T> {
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

pub trait Spawner<'a> {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'a;
}

pub trait LocalSpawner<'a> {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'a;
}

pub trait BlockingSpawner {
    type Future<R>: Future<Output = Result<R, Self::Error>>;
    type Error;
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static;
}

pub trait HasSpawner<'a> {
    type Spawner: Spawner<'a>;
    fn spawner(&self) -> &Self::Spawner;
}

pub trait HasLocalSpawner<'a> {
    type Spawner: LocalSpawner<'a>;
    fn local_spawner(&self) -> &Self::Spawner;
}

pub trait HasBlockingSpawner {
    type Spawner: BlockingSpawner;
    fn blocking_spawner(&self) -> &Self::Spawner;
}

pub struct LocalExecutor<'js, R = ()> {
    inner: Rc<dyn Executor<LocalBoxFuture<'js, R>> + 'js>,
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
    pub fn new<T>(inner: T) -> Self
    where
        T: Executor<LocalBoxFuture<'js, R>> + 'js,
    {
        LocalExecutor {
            inner: Rc::from(inner),
        }
    }

    pub fn spawn<T>(&self, work: T)
    where
        T: Future<Output = R> + 'js,
    {
        self.inner.spawn(Box::pin(work));
    }
}

pub struct SendExecutor<'js, R = ()> {
    inner: Arc<dyn Executor<BoxFuture<'js, R>> + Send + Sync + 'js>,
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
    pub fn new<T>(inner: T) -> Self
    where
        T: Executor<BoxFuture<'js, R>> + 'js + Send + Sync,
    {
        SendExecutor {
            inner: Arc::from(inner),
        }
    }

    pub fn spawn<T>(&self, work: T)
    where
        T: Future<Output = R> + Send + 'js,
    {
        self.inner.spawn(Box::pin(work));
    }
}

impl<'js, T, R> Executor<T> for LocalExecutor<'js, R>
where
    T: Future<Output = R> + 'js,
{
    fn spawn(&self, work: T) {
        self.inner.spawn(Box::pin(work));
    }
}

impl<'js, T, R> Executor<T> for SendExecutor<'js, R>
where
    T: Future<Output = R> + Send + 'js,
{
    fn spawn(&self, work: T) {
        self.inner.spawn(Box::pin(work));
    }
}

#[cfg(feature = "hyper")]
impl<'js, T, R> hyper::rt::Executor<T> for LocalExecutor<'js, R>
where
    T: Future<Output = R> + 'js,
{
    fn execute(&self, work: T) {
        self.inner.spawn(Box::pin(work));
    }
}

#[cfg(feature = "hyper")]
impl<'js, T, R> hyper::rt::Executor<T> for SendExecutor<'js, R>
where
    T: Future<Output = R> + Send + 'js,
{
    fn execute(&self, work: T) {
        self.inner.spawn(Box::pin(work));
    }
}

#[cfg(feature = "tokio")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokioExecutor;

#[cfg(feature = "tokio")]
impl<T> Executor<T> for TokioExecutor
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn spawn(&self, work: T) {
        tokio::task::spawn(work);
    }
}

#[cfg(feature = "tokio")]
impl Spawner<'static> for TokioExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'static,
    {
        tokio::task::spawn(work);
    }
}

#[cfg(feature = "tokio")]
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

#[cfg(feature = "tokio")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LocalTokioExecutor;

#[cfg(feature = "tokio")]
impl<T> Executor<T> for LocalTokioExecutor
where
    T: Future + 'static,
    T::Output: 'static,
{
    fn spawn(&self, work: T) {
        tokio::task::spawn_local(work);
    }
}

#[cfg(feature = "tokio")]
impl Spawner<'static> for LocalTokioExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        tokio::task::spawn_local(work);
    }
}

#[cfg(feature = "tokio")]
impl LocalSpawner<'static> for LocalTokioExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        tokio::task::spawn_local(work);
    }
}

#[cfg(feature = "tokio")]
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

#[cfg(all(feature = "tokio", feature = "hyper"))]
impl<T> hyper::rt::Executor<T> for TokioExecutor
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn execute(&self, work: T) {
        tokio::task::spawn(work);
    }
}

#[cfg(all(feature = "tokio", feature = "hyper"))]
impl<T> hyper::rt::Executor<T> for LocalTokioExecutor
where
    T: Future + 'static,
{
    fn execute(&self, work: T) {
        tokio::task::spawn_local(work);
    }
}

#[cfg(feature = "smol")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SmolExecutor;

#[cfg(feature = "smol")]
impl<T> Executor<T> for SmolExecutor
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn spawn(&self, work: T) {
        smol::spawn(work).detach();
    }
}

#[cfg(all(feature = "smol", feature = "hyper"))]
impl<T> hyper::rt::Executor<T> for SmolExecutor
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    fn execute(&self, work: T) {
        smol::spawn(work).detach();
    }
}

#[cfg(feature = "smol")]
impl Spawner<'static> for SmolExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'static,
    {
        smol::spawn(work).detach();
    }
}

#[cfg(feature = "smol")]
impl BlockingSpawner for SmolExecutor {
    type Error = core::convert::Infallible;
    type Future<R> = SmolBlockingFuture<R>;
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        SmolBlockingFuture(smol::unblock(work))
    }
}

#[cfg(feature = "smol")]
#[derive(Debug)]
pub struct SmolBlockingFuture<R>(smol::Task<R>);

#[cfg(feature = "smol")]
impl<R> core::future::Future for SmolBlockingFuture<R> {
    type Output = Result<R, core::convert::Infallible>;
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx).map(Ok)
    }
}

#[cfg(feature = "smol")]
impl HasBlockingSpawner for SmolExecutor {
    type Spawner = Self;
    fn blocking_spawner(&self) -> &Self::Spawner {
        self
    }
}

#[cfg(feature = "smol")]
impl HasSpawner<'static> for SmolExecutor {
    type Spawner = Self;
    fn spawner(&self) -> &Self::Spawner {
        self
    }
}

#[cfg(feature = "compio")]
#[derive(Debug, Clone, Copy, Default)]
pub struct CompioExecutor;

#[cfg(feature = "compio")]
impl<T> Executor<T> for CompioExecutor
where
    T: Future + 'static,
    T::Output: 'static,
{
    fn spawn(&self, work: T) {
        compio::runtime::spawn(work).detach();
    }
}

#[cfg(all(feature = "compio", feature = "hyper"))]
impl<T> hyper::rt::Executor<T> for CompioExecutor
where
    T: Future + 'static,
    T::Output: 'static,
{
    fn execute(&self, work: T) {
        compio::runtime::spawn(work).detach();
    }
}

#[cfg(feature = "compio")]
impl Spawner<'static> for CompioExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + Send + 'static,
    {
        compio::runtime::spawn(work).detach();
    }
}

#[cfg(feature = "compio")]
impl LocalSpawner<'static> for CompioExecutor {
    fn spawn<T>(&self, work: T)
    where
        T: core::future::Future<Output = ()> + 'static,
    {
        compio::runtime::spawn(work).detach();
    }
}

#[cfg(feature = "compio")]
impl BlockingSpawner for CompioExecutor {
    type Error = compio_executor::JoinError;
    type Future<R> = compio_executor::JoinHandle<R>;
    fn spawn_blocking<T, R>(&self, work: T) -> Self::Future<R>
    where
        R: Send + 'static,
        T: FnOnce() -> R + Send + 'static,
    {
        compio::runtime::spawn_blocking(work)
    }
}

#[cfg(feature = "compio")]
impl HasBlockingSpawner for CompioExecutor {
    type Spawner = Self;
    fn blocking_spawner(&self) -> &Self::Spawner {
        self
    }
}

#[cfg(feature = "compio")]
impl HasLocalSpawner<'static> for CompioExecutor {
    type Spawner = Self;
    fn local_spawner(&self) -> &Self::Spawner {
        self
    }
}

#[cfg(feature = "compio")]
impl HasSpawner<'static> for CompioExecutor {
    type Spawner = Self;
    fn spawner(&self) -> &Self::Spawner {
        self
    }
}
