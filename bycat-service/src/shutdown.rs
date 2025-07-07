use alloc::sync::Arc;
use core::{
    fmt::Debug,
    pin::Pin,
    task::{Context, Poll},
};
use event_listener::{Event, EventListener};
use pin_project_lite::pin_project;

#[derive(Clone)]
pub struct Shutdown {
    event: Arc<Event>,
}

impl Debug for Shutdown {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Shutdown").finish_non_exhaustive()
    }
}

impl Shutdown {
    pub fn new() -> Self {
        Shutdown {
            event: Event::new().into(),
        }
    }

    pub fn watch<C: GracefulShutdown>(&self, conn: C) -> GracefulWatchFuture<C> {
        GracefulWatchFuture {
            conn,
            cancel: self.event.listen(),
            guard: None,
        }
    }

    pub async fn shutdown(&self) {
        self.event.notify(usize::MAX);
    }
}

pub trait GracefulShutdown: Future<Output = Result<(), Self::Error>> {
    type Error;

    fn graceful_shutdown(self: Pin<&mut Self>);
}

pin_project! {
  pub struct GracefulWatchFuture<C: GracefulShutdown> {
    #[pin]
    conn: C,
    #[pin]
    cancel: EventListener,
    #[pin]
    guard: Option<()>
  }
}

impl<C> Future for GracefulWatchFuture<C>
where
    C: GracefulShutdown,
{
    type Output = C::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        if this.guard.is_none() {
            if let Poll::Ready(guard) = this.cancel.poll(cx) {
                this.guard.set(Some(guard));
                this.conn.as_mut().graceful_shutdown();
            }
        }

        this.conn.poll(cx)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use core::cell::Cell;
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    use futures::executor::block_on;

    struct TestConn {
        polled: Rc<Cell<u32>>,
        shutdown_called: Rc<Cell<bool>>,
        ready: bool,
    }

    impl Future for TestConn {
        type Output = Result<(), &'static str>;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = self.get_mut();
            this.polled.set(this.polled.get() + 1);
            if this.ready {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }
    }

    impl GracefulShutdown for TestConn {
        type Error = &'static str;

        fn graceful_shutdown(self: Pin<&mut Self>) {
            let this = self.get_mut();
            this.shutdown_called.set(true);
            this.ready = true;
        }
    }

    fn dummy_waker() -> Waker {
        fn no_op(_: *const ()) {}
        fn clone(_: *const ()) -> RawWaker {
            dummy_raw_waker()
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, no_op, no_op, no_op);
        fn dummy_raw_waker() -> RawWaker {
            RawWaker::new(core::ptr::null(), &VTABLE)
        }
        unsafe { Waker::from_raw(dummy_raw_waker()) }
    }

    #[test]
    fn test_shutdown_triggers_graceful_shutdown() {
        let shutdown = Shutdown::new();
        let polled = Rc::new(Cell::new(0));
        let shutdown_called = Rc::new(Cell::new(false));
        let conn = TestConn {
            polled: polled.clone(),
            shutdown_called: shutdown_called.clone(),
            ready: false,
        };

        let mut fut = shutdown.watch(conn);

        // Poll once, should be pending and not shutdown
        let waker = dummy_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(Pin::new(&mut fut).poll(&mut cx).is_pending());
        assert_eq!(polled.get(), 1);
        assert!(!shutdown_called.get());

        // Trigger shutdown
        block_on(shutdown.shutdown());

        // Now, polling should call graceful_shutdown and complete
        assert!(Pin::new(&mut fut).poll(&mut cx).is_ready());
        assert!(shutdown_called.get());
    }

    #[test]
    fn test_multiple_shutdowns() {
        let shutdown = Shutdown::new();
        let polled = Rc::new(Cell::new(0));
        let shutdown_called = Rc::new(Cell::new(false));
        let conn = TestConn {
            polled: polled.clone(),
            shutdown_called: shutdown_called.clone(),
            ready: false,
        };

        let mut fut = shutdown.watch(conn);

        // Poll once, should be pending
        let waker = dummy_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(Pin::new(&mut fut).poll(&mut cx).is_pending());

        // Trigger shutdown multiple times
        block_on(shutdown.shutdown());
        block_on(shutdown.shutdown());

        // Should still only call graceful_shutdown once and complete
        assert!(Pin::new(&mut fut).poll(&mut cx).is_ready());
        assert!(shutdown_called.get());
    }

    #[test]
    fn test_shutdown_clone() {
        let shutdown = Shutdown::new();
        let shutdown2 = shutdown.clone();
        let polled = Rc::new(Cell::new(0));
        let shutdown_called = Rc::new(Cell::new(false));
        let conn = TestConn {
            polled: polled.clone(),
            shutdown_called: shutdown_called.clone(),
            ready: false,
        };

        let mut fut = shutdown2.watch(conn);

        let waker = dummy_waker();
        let mut cx = Context::from_waker(&waker);
        assert!(Pin::new(&mut fut).poll(&mut cx).is_pending());

        // Trigger shutdown from original
        block_on(shutdown.shutdown());

        assert!(Pin::new(&mut fut).poll(&mut cx).is_ready());
        assert!(shutdown_called.get());
    }
}
