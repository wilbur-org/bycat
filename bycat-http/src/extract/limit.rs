use core::task::{Poll, ready};

use bycat::{Middleware, Work};
use bytes::Bytes;
use http::{Request, Response, StatusCode, header::CONTENT_LENGTH};
use pin_project_lite::pin_project;

use crate::{IntoResponse, body::HttpBody};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestBodyLimit(pub u64);

impl<C, B, T> Middleware<C, Request<B>, T> for RequestBodyLimit
where
    T: Work<C, Request<B>>,
    T::Output: IntoResponse<B>,
    B: HttpBody + Send + Sync + 'static,
    B::Data: AsRef<[u8]> + Into<Bytes>,
    B::Error: core::error::Error + Send + Sync + 'static,
{
    type Work = RequestBodyLimitWork<T>;

    fn wrap(&self, handle: T) -> Self::Work {
        RequestBodyLimitWork(handle, self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RequestBodyLimitWork<T>(T, u64);

impl<T, C, B> Work<C, Request<B>> for RequestBodyLimitWork<T>
where
    T: Work<C, Request<B>>,
    T::Output: IntoResponse<B>,
    B: HttpBody + Send + Sync + 'static,
    B::Data: AsRef<[u8]> + Into<Bytes>,
    B::Error: core::error::Error + Send + Sync + 'static,
{
    type Output = RequestBodyLimitWorkResponse<T::Output, B>;

    type Error = T::Error;

    type Future<'a>
        = RequestBodyLimitWorkFuture<'a, T, C, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        RequestBodyLimitWorkFuture {
            state: RequestBodyLimitWorkState::Init {
                context,
                work: &self.0,
                limit: self.1,
                request: Some(req),
            },
        }
    }
}

pin_project! {
    #[project = RequestBodyLimitWorkStateProj]
    enum RequestBodyLimitWorkState<'a, T, C, B>
    where
        T: Work<C, Request<B>>,
    {
        Init {
            context: &'a C,
            work: &'a T,
            limit: u64,
            request: Option<Request<B>>,
        },
        Future {
            #[pin]
            future: T::Future<'a>,
        },
        Done,
    }
}

pin_project! {
    pub struct RequestBodyLimitWorkFuture<'a, T, C, B>
    where
        T: Work<C, Request<B>>,
    {
        #[pin]
        state: RequestBodyLimitWorkState<'a, T, C, B>
    }
}

impl<'a, T, C, B> Future for RequestBodyLimitWorkFuture<'a, T, C, B>
where
    T: Work<C, Request<B>>,
    T::Output: IntoResponse<B>,
    B: HttpBody + Send + Sync + 'static,
    B::Data: AsRef<[u8]> + Into<Bytes>,
    B::Error: core::error::Error + Send + Sync + 'static,
{
    type Output = Result<RequestBodyLimitWorkResponse<T::Output, B>, T::Error>;

    fn poll(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                RequestBodyLimitWorkStateProj::Init {
                    context,
                    work,
                    request,
                    limit,
                } => {
                    let request = request.take().unwrap();

                    if let Some(content_len) = request.headers().get(CONTENT_LENGTH)
                        && let Ok(str) = content_len.to_str()
                        && let Ok(len) = u64::from_str_radix(str, 10)
                    {
                        if len > *limit {
                            let mut resp = Response::new(B::empty());

                            *resp.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                            this.state.set(RequestBodyLimitWorkState::Done);
                            return Poll::Ready(Ok(RequestBodyLimitWorkResponse::Error(resp)));
                        }
                    }

                    let future = work.call(
                        context,
                        request.map(|body| {
                            B::from_streaming(RequestBodyLimitBody {
                                limit: *limit as _,
                                read: 0,
                                body,
                            })
                        }),
                    );

                    this.state.set(RequestBodyLimitWorkState::Future { future });
                }
                RequestBodyLimitWorkStateProj::Future { future } => {
                    let ret = ready!(future.poll(cx));
                    this.state.set(RequestBodyLimitWorkState::Done);
                    return Poll::Ready(ret.map(RequestBodyLimitWorkResponse::Ok));
                }
                RequestBodyLimitWorkStateProj::Done => {
                    panic!("Poll after done")
                }
            }
        }
    }
}

pub enum RequestBodyLimitWorkResponse<T, B> {
    Error(Response<B>),
    Ok(T),
}

impl<T, B> IntoResponse<B> for RequestBodyLimitWorkResponse<T, B>
where
    T: IntoResponse<B>,
{
    type Error = T::Error;

    fn into_response(self) -> Result<Response<B>, T::Error> {
        match self {
            Self::Error(err) => Ok(err),
            Self::Ok(ret) => ret.into_response(),
        }
    }
}

pin_project! {
    pub struct RequestBodyLimitBody<T> {
        #[pin]
        body: T,
        limit: usize,
        read: usize,
    }

}

impl<T> http_body::Body for RequestBodyLimitBody<T>
where
    T: http_body::Body,
    T::Data: AsRef<[u8]>,
{
    type Data = T::Data;

    type Error = T::Error;

    fn poll_frame(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        if this.read >= this.limit {
            return Poll::Ready(None);
        }

        match ready!(this.body.poll_frame(cx)) {
            Some(Ok(frame)) => {
                if let Some(data) = frame.data_ref() {
                    *this.limit += data.as_ref().len();
                }

                Poll::Ready(Some(Ok(frame)))
            }
            Some(Err(err)) => Poll::Ready(Some(Err(err))),
            None => Poll::Ready(None),
        }
    }
}
