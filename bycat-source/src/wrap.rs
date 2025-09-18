use core::marker::PhantomData;

use alloc::boxed::Box;
use arbejd::Work;
use futures::{future::BoxFuture, TryFuture, TryFutureExt};

use crate::Error;

#[derive(Debug)]
pub struct Wrap<T, F, C> {
    task: T,
    func: F,
    ctx: PhantomData<C>,
}

impl<T: Clone, F: Clone, C> Clone for Wrap<T, F, C> {
    fn clone(&self) -> Self {
        Wrap {
            task: self.task.clone(),
            func: self.func.clone(),
            ctx: PhantomData,
        }
    }
}

impl<T: Copy, F: Copy, C> Copy for Wrap<T, F, C> {}

unsafe impl<T: Send, F: Send, C> Send for Wrap<T, F, C> {}

unsafe impl<T: Sync, F: Sync, C> Sync for Wrap<T, F, C> {}

impl<T, F, C> Wrap<T, F, C> {
    pub fn new(task: T, func: F) -> Wrap<T, F, C> {
        Wrap {
            task,
            func,
            ctx: PhantomData,
        }
    }
}

impl<T, F, U, C: Clone, R> Work<C, R> for Wrap<T, F, C>
where
    T: Work<C, R> + Clone + Send + 'static,
    F: Fn(C, R, T) -> U + Clone + Send + 'static,
    U: TryFuture + Send,
    C: Send + 'static,
    R: Send + 'static,
{
    type Error = U::Error;
    type Output = U::Ok;
    type Future<'a> = BoxFuture<'a, Result<U::Ok, U::Error>>;

    fn call<'a>(&'a self, ctx: &'a C, package: R) -> Self::Future<'a> {
        let work = self.task.clone();
        let func = self.func.clone();
        Box::pin(async move { (func)(ctx.clone(), package, work).await })
    }
}
