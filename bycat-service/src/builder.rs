use alloc::vec::Vec;
use bycat_error::Error;
use core::task::{Poll, ready};
use futures::{Stream, stream::FuturesUnordered};
use pin_project_lite::pin_project;

use crate::{service::Service, shutdown::Shutdown};

#[derive(Debug, Clone)]
pub struct ServiceBuilder<S> {
    service: Vec<S>,
}

impl<S> Default for ServiceBuilder<S> {
    fn default() -> Self {
        ServiceBuilder {
            service: Default::default(),
        }
    }
}

impl<S> ServiceBuilder<S>
where
    S: Service,
{
    pub fn push(&mut self, service: S) -> &mut Self {
        self.service.push(service);
        self
    }

    pub async fn serve<'a>(&'a self, shutdown: &'a Shutdown) -> ServiceFuture<'a, S> {
        let futures = FuturesUnordered::new();

        for service in &self.service {
            futures.push(service.serve(shutdown));
        }

        ServiceFuture { futures }
    }
}

pin_project! {
    pub struct ServiceFuture<'a, S: 'a>
    where
        S: Service,
    {
        #[pin]
        futures: FuturesUnordered<S::Future<'a>>,
    }

}

impl<'a, S: 'a> Future for ServiceFuture<'a, S>
where
    S: Service,
{
    type Output = Result<(), Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();
            match ready!(this.futures.as_mut().poll_next(cx)) {
                Some(Ok(_)) => {
                    if this.futures.is_empty() {
                        return Poll::Ready(Ok(()));
                    }
                    continue;
                }
                Some(Err(err)) => return Poll::Ready(Err(err)),
                None => {
                    if this.futures.is_empty() {
                        return Poll::Ready(Ok(()));
                    }
                }
            }
        }
    }
}
