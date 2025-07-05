use core::task::{Poll, ready};

use either::Either;
use futures_core::Stream;
use pin_project_lite::pin_project;

pub trait IntoResult {
    type Output;
    type Error;

    fn into_result(self) -> Result<Self::Output, Self::Error>;
}

impl<T, E> IntoResult for Result<T, E> {
    type Error = E;
    type Output = T;
    fn into_result(self) -> Result<Self::Output, Self::Error> {
        self
    }
}

impl<L, R> IntoResult for Either<L, R>
where
    L: IntoResult,
    R: IntoResult<Output = L::Output, Error = L::Error>,
{
    type Error = L::Error;
    type Output = L::Output;
    fn into_result(self) -> Result<Self::Output, Self::Error> {
        match self {
            Either::Left(left) => left.into_result(),
            Either::Right(right) => right.into_result(),
        }
    }
}

macro_rules! into_result {
    ($first: ident) => {
        impl<$first> IntoResult for ($first,)
        where
            $first: IntoResult,
        {
            type Output = ($first::Output,);
            type Error = $first::Error;

            fn into_result(self) -> Result<Self::Output, Self::Error> {
                self.0.into_result().map(|m| (m,))
            }
        }
    };
    ($first: ident, $($rest: ident),+) => {
      into_result!($($rest),+);

      #[allow(non_snake_case)]
      impl<$first, $($rest),+> IntoResult for ($first, $($rest),+)
      where
        $first: IntoResult,
        $(
          $rest: IntoResult,
          $rest::Error: Into<$first::Error>
        ),+
      {
        type Output = ($first::Output, $($rest::Output),+);
        type Error = $first::Error;

        fn into_result(self) -> Result<Self::Output, Self::Error> {
          let ($first, $($rest),+) = self;

          Ok(
            (
              $first.into_result()?,
              $(
                $rest.into_result().map_err(Into::into)?
              ),+
            )
          )
        }
      }
    };
}

into_result!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

pub trait IteratorResultExt: Iterator {
    fn into_result(self) -> ResultIterator<Self>
    where
        Self: Sized,
        Self::Item: IntoResult,
    {
        ResultIterator(self)
    }
}

impl<T> IteratorResultExt for T where T: Iterator {}

pub struct ResultIterator<T>(pub T);

impl<T> ResultIterator<T> {
    pub fn new(iter: T) -> ResultIterator<T> {
        ResultIterator(iter)
    }
}

impl<T> Iterator for ResultIterator<T>
where
    T: Iterator,
    T::Item: IntoResult,
{
    type Item = Result<<T::Item as IntoResult>::Output, <T::Item as IntoResult>::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|item| item.into_result())
    }
}

pub trait StreamResultExt: Stream {
    fn into_result(self) -> ResultStream<Self>
    where
        Self: Sized,
        Self::Item: IntoResult,
    {
        ResultStream::new(self)
    }
}

impl<T> StreamResultExt for T where T: Stream {}

pin_project! {
  pub struct ResultStream<T> {
      #[pin]
      stream: T
  }
}

impl<T> ResultStream<T> {
    pub fn new(stream: T) -> ResultStream<T> {
        ResultStream { stream }
    }
}

impl<T> Stream for ResultStream<T>
where
    T: Stream,
    T::Item: IntoResult,
{
    type Item = Result<<T::Item as IntoResult>::Output, <T::Item as IntoResult>::Error>;

    fn poll_next(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<<Self as Stream>::Item>> {
        match ready!(self.project().stream.poll_next(cx)) {
            Some(item) => Poll::Ready(Some(item.into_result())),
            None => Poll::Ready(None),
        }
    }
}

pub trait FutureResultExt: Future {
    fn into_result(self) -> ResultFuture<Self>
    where
        Self: Sized,
        Self::Output: IntoResult,
    {
        ResultFuture::new(self)
    }
}

impl<T> FutureResultExt for T where T: Future {}

pin_project! {
  pub struct ResultFuture<T> {
      #[pin]
      future: T
  }
}

impl<T> ResultFuture<T> {
    pub fn new(future: T) -> ResultFuture<T> {
        ResultFuture { future }
    }
}

impl<T> Future for ResultFuture<T>
where
    T: Future,
    T::Output: IntoResult,
{
    type Output = Result<<T::Output as IntoResult>::Output, <T::Output as IntoResult>::Error>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        Poll::Ready(ready!(self.project().future.poll(cx)).into_result())
    }
}
