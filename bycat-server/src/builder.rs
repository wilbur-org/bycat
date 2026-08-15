use alloc::vec::Vec;
use core::task::{Poll, ready};
use futures::{Stream, stream::FuturesUnordered};
use pin_project_lite::pin_project;

use crate::{service::Service, shutdown::Shutdown};

#[derive(Debug)]
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

    pub fn serve<'a>(&'a self, shutdown: &'a Shutdown) -> ServiceFuture<'a, S> {
        let futures = FuturesUnordered::new();

        for service in &self.service {
            futures.push(service.serve(shutdown));
        }

        ServiceFuture { futures }
    }
}

impl<S> Service for ServiceBuilder<S>
where
    S: Service,
{
    type Error = S::Error;
    type Future<'a>
        = ServiceFuture<'a, S>
    where
        Self: 'a,
        S: 'a;
    fn serve<'a>(&'a self, shutdown: &'a Shutdown) -> Self::Future<'a> {
        self.serve(shutdown)
    }
}

pin_project! {
    pub struct ServiceFuture<'a, S: 'a>
    where
        S: Service
    {
        #[pin]
        futures: FuturesUnordered<S::Future<'a>>,
    }

}

impl<'a, S: 'a> Future for ServiceFuture<'a, S>
where
    S: Service,
{
    type Output = Result<(), S::Error>;

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
#[cfg(test)]
mod tests {
    use super::*;

    use alloc::rc::Rc;
    use core::cell::RefCell;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};
    use futures::executor::block_on;

    struct TestService {
        result: Result<(), &'static str>,
        called: Rc<RefCell<bool>>,
    }

    impl Service for TestService {
        type Error = &'static str;
        type Future<'a>
            = TestServiceFuture
        where
            Self: 'a;

        fn serve<'a>(&'a self, _shutdown: &'a Shutdown) -> Self::Future<'a> {
            *self.called.borrow_mut() = true;
            TestServiceFuture {
                result: self.result.clone(),
            }
        }
    }

    struct TestServiceFuture {
        result: Result<(), &'static str>,
    }

    impl Future for TestServiceFuture {
        type Output = Result<(), &'static str>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Ready(self.result.clone())
        }
    }

    #[test]
    fn test_service_builder_push_and_serve_success() {
        let called = Rc::new(RefCell::new(false));
        let service = TestService {
            result: Ok(()),
            called: called.clone(),
        };
        let mut builder = ServiceBuilder::default();
        builder.push(service);

        let shutdown = Shutdown::new();
        let result = block_on(builder.serve(&shutdown));

        assert!(result.is_ok());
        assert!(*called.borrow_mut());
    }

    #[test]
    fn test_service_builder_multiple_services() {
        let called1 = Rc::new(RefCell::new(false));
        let called2 = Rc::new(RefCell::new(false));
        let service1 = TestService {
            result: Ok(()),
            called: called1.clone(),
        };
        let service2 = TestService {
            result: Ok(()),
            called: called2.clone(),
        };
        let mut builder = ServiceBuilder::default();
        builder.push(service1);
        builder.push(service2);

        let shutdown = Shutdown::new();
        let result = block_on(builder.serve(&shutdown));

        assert!(result.is_ok());
        assert!(*called1.borrow_mut());
        assert!(*called2.borrow_mut());
    }

    #[test]
    fn test_service_builder_error_propagation() {
        let called1 = Rc::new(RefCell::new(false));
        let called2 = Rc::new(RefCell::new(false));
        let service1 = TestService {
            result: Ok(()),
            called: called1.clone(),
        };
        let service2 = TestService {
            result: Err("fail"),
            called: called2.clone(),
        };
        let mut builder = ServiceBuilder::default();
        builder.push(service1);
        builder.push(service2);

        let shutdown = Shutdown::new();
        let result = block_on(builder.serve(&shutdown));

        assert!(result.is_err());
        assert!(*called1.borrow_mut());
        assert!(*called2.borrow_mut());
    }

    #[test]
    fn test_service_builder_empty() {
        let builder: ServiceBuilder<TestService> = ServiceBuilder::default();
        let shutdown = Shutdown::new();
        let result = block_on(builder.serve(&shutdown));
        assert!(result.is_ok());
    }
}
