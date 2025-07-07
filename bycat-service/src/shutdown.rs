use alloc::sync::Arc;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use event_listener::{Event, EventListener};
use pin_project_lite::pin_project;

#[derive(Clone)]
pub struct Shutdown {
    event: Arc<Event>,
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
