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

pub struct LocalExecutor<'js, R = ()> {
    inner: Rc<dyn Executor<LocalBoxFuture<'js, R>> + 'js>,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
