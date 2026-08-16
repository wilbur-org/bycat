use crate::Service;
use core::task::Poll;
use futures_core::{TryFuture, ready};
use pin_project_lite::pin_project;

pub fn service_fn<T, C, R, U>(func: T) -> ServiceFn<T>
where
    T: Fn(C, R) -> U,
    U: TryFuture,
    C: Clone,
{
    ServiceFn(func)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServiceFn<T>(pub(crate) T);

impl<T, U, C, R> Service<C, R> for ServiceFn<T>
where
    T: Fn(C, R) -> U,
    U: TryFuture,
    C: Clone,
{
    type Output = U::Ok;
    type Error = U::Error;
    type Future<'a>
        = ServiceFnFuture<U>
    where
        Self: 'a,
        C: 'a;
    fn call<'this: 'lifetime, 'ctx: 'lifetime, 'lifetime>(
        &'this self,
        ctx: &'ctx C,
        package: R,
    ) -> Self::Future<'lifetime> {
        ServiceFnFuture {
            future: (self.0)(ctx.clone(), package),
        }
    }
}

pin_project! {
  pub struct ServiceFnFuture<U> {
    #[pin]
    future: U
  }
}

impl<U> Future for ServiceFnFuture<U>
where
    U: TryFuture,
{
    type Output = Result<U::Ok, U::Error>;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let this = self.project();
        match ready!(this.future.try_poll(cx)) {
            Ok(ret) => Poll::Ready(Ok(ret)),
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
