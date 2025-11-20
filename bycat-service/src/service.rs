use core::task::Poll;

use pin_project_lite::pin_project;

use crate::shutdown::Shutdown;

pub trait Service {
    type Error;
    type Future<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;
    fn serve<'a>(&'a self, shutdown: &'a Shutdown) -> Self::Future<'a>;
}

pub trait ServiceExt: Service {
    fn and<T>(self, service: T) -> AndService<Self, T>
    where
        Self: Sized,
    {
        AndService(self, service)
    }
}

impl<T> ServiceExt for T where T: Service {}

pub struct AndService<T1, T2>(T1, T2);

impl<T1, T2> AndService<T1, T2> {
    pub fn new(t1: T1, t2: T2) -> AndService<T1, T2> {
        AndService(t1, t2)
    }
}

impl<T1, T2> Service for AndService<T1, T2>
where
    T1: Service,
    T2: Service,
    T2::Error: Into<T1::Error>,
{
    type Error = T1::Error;

    type Future<'a>
        = AndServiceFuture<'a, T1, T2>
    where
        Self: 'a;

    fn serve<'a>(&'a self, shutdown: &'a Shutdown) -> Self::Future<'a> {
        AndServiceFuture {
            t1: State::Future {
                future: self.0.serve(shutdown),
            },
            t2: State::Future {
                future: self.1.serve(shutdown),
            },
        }
    }
}

pin_project! {
    #[project = StateProj]
    enum State<T>
    where
        T: Future,
    {
        Future {
            #[pin]
            future: T
        },
        Done
    }
}

pin_project! {
    pub struct AndServiceFuture<'a, T1: 'a, T2: 'a>
where
    T1: Service,
    T2: Service,
    T2::Error: Into<T1::Error>,
{
    #[pin]
    t1: State<T1::Future<'a>>,
    #[pin]
    t2: State<T2::Future<'a>>,
}
}

impl<'a, T1: 'a, T2: 'a> Future for AndServiceFuture<'a, T1, T2>
where
    T1: Service,
    T2: Service,
    T2::Error: Into<T1::Error>,
{
    type Output = Result<(), T1::Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let first = match this.t1.as_mut().project() {
            StateProj::Future { future } => match future.poll(cx) {
                Poll::Pending => false,
                Poll::Ready(Ok(_)) => {
                    this.t1.set(State::Done);
                    true
                }
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            },
            StateProj::Done => true,
        };

        let last = match this.t2.as_mut().project() {
            StateProj::Future { future } => match future.poll(cx) {
                Poll::Pending => false,
                Poll::Ready(Ok(_)) => {
                    this.t2.set(State::Done);
                    true
                }
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err.into())),
            },
            StateProj::Done => true,
        };

        match (first, last) {
            (true, true) => Poll::Ready(Ok(())),
            _ => Poll::Pending,
        }
    }
}
