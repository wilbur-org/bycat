use alloc::{
    fmt::{self, Debug},
    marker::PhantomData,
    sync::Arc,
    task::{Poll, ready},
    time::Duration,
};
use bycat::{Middleware, Work};
use bycat_error::{BoxError, Error};
use http::{
    HeaderMap, HeaderValue, Method, Request, Response,
    header::{
        ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_METHODS,
        ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_METHOD,
    },
};
use pin_project_lite::pin_project;
use routing::router::MethodFilter;
use std::fmt::Write as _;

use crate::IntoResponse;

#[derive(Debug, Clone)]
pub enum AllowedOrigins {
    Any,
    Origin(HeaderValue),
    Request,
}

impl AllowedOrigins {
    fn apply(&self, headers: &mut HeaderMap) {
        match self {
            AllowedOrigins::Any => {
                headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
            }
            AllowedOrigins::Origin(orig) => {
                headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, orig.clone());
            }
            AllowedOrigins::Request => {
                // When echoing the request origin is desired, it must be handled
                // where the request is available; do nothing here.
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum AllowedHeaders {
    Any,
    List(Vec<String>),
    Request,
}

impl AllowedHeaders {
    pub fn apply(&self, headers: &mut HeaderMap) {
        match self {
            AllowedHeaders::Any => {
                headers.insert(
                    http::header::ACCESS_CONTROL_ALLOW_HEADERS,
                    HeaderValue::from_static("*"),
                );
            }
            AllowedHeaders::List(list) => {
                if let Ok(value) = HeaderValue::from_str(&list.join(", ")) {
                    headers.insert(http::header::ACCESS_CONTROL_ALLOW_HEADERS, value);
                }
            }
            AllowedHeaders::Request => {
                // When echoing the request origin is desired, it must be handled
                // where the request is available; do nothing here.
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CorsOptions {
    origins: AllowedOrigins,
    headers: AllowedHeaders,
    methods: MethodFilter,
    max_age: Option<Duration>,
    credentials: bool,
}

impl Default for CorsOptions {
    fn default() -> Self {
        CorsOptions {
            origins: AllowedOrigins::Request,
            headers: AllowedHeaders::Request,
            methods: MethodFilter::GET
                | MethodFilter::POST
                | MethodFilter::PUT
                | MethodFilter::PATCH
                | MethodFilter::DELETE,
            max_age: None,
            credentials: false,
        }
    }
}

impl CorsOptions {
    fn apply(&self, headers: &mut HeaderMap, preflight: bool) {
        self.origins.apply(headers);

        if preflight {
            self.headers.apply(headers);

            let mut methods = String::new();

            for (idx, method) in self.methods.iter().enumerate() {
                if idx > 0 {
                    methods.push(',');
                }
                write!(methods, "{}", method).expect("Write");
            }

            headers.insert(
                ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_str(&methods).expect("HeaderValue"),
            );
        }

        if self.credentials {
            headers.insert(
                ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            );
        }

        if let Some(max_age) = self.max_age {
            if let Ok(value) = HeaderValue::from_str(&max_age.as_secs().to_string()) {
                headers.insert(http::header::ACCESS_CONTROL_MAX_AGE, value);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cors {
    opts: Arc<CorsOptions>,
}

impl Default for Cors {
    fn default() -> Self {
        Cors::new()
    }
}

impl Cors {
    pub fn new() -> Cors {
        Cors {
            opts: Arc::new(Default::default()),
        }
    }

    pub fn allowed_methods(mut self, methods: MethodFilter) -> Self {
        Arc::make_mut(&mut self.opts).methods = methods;
        self
    }

    pub fn allowed_headers(mut self, headers: AllowedHeaders) -> Self {
        Arc::make_mut(&mut self.opts).headers = headers;
        self
    }

    pub fn allowed_origin(mut self, origin: AllowedOrigins) -> Self {
        Arc::make_mut(&mut self.opts).origins = origin;
        self
    }
}

impl<T, C, B> Middleware<C, Request<B>, T> for Cors
where
    T: Work<C, Request<B>>,
    T::Error: Into<BoxError>,
    T::Output: IntoResponse<B>,
    <T::Output as IntoResponse<B>>::Error: Into<BoxError>,
{
    type Work = CorsWork<T, C, B>;

    fn wrap(&self, handle: T) -> Self::Work {
        CorsWork {
            opts: self.opts.clone(),
            work: handle,
            ph: PhantomData,
        }
    }
}

pub struct CorsWork<T, C, B> {
    opts: Arc<CorsOptions>,
    work: T,
    ph: PhantomData<fn() -> (C, B)>,
}

impl<T: Clone, C, B> Clone for CorsWork<T, C, B> {
    fn clone(&self) -> Self {
        Self {
            opts: self.opts.clone(),
            work: self.work.clone(),
            ph: PhantomData,
        }
    }
}

impl<T: Debug, C, B> fmt::Debug for CorsWork<T, C, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorsWork")
            .field("opts", &self.opts)
            .field("work", &self.work)
            .finish()
    }
}

impl<T, C, B> Work<C, Request<B>> for CorsWork<T, C, B>
where
    T: Work<C, Request<B>>,
    T::Error: Into<BoxError>,
    T::Output: IntoResponse<B>,
    <T::Output as IntoResponse<B>>::Error: Into<BoxError>,
{
    type Output = Response<B>;

    type Error = Error;

    type Future<'a>
        = CorsFuture<'a, C, T, B>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        CorsFuture {
            state: CorsFutureState::Init {
                work: &self.work,
                context,
                req: Some(req),
            },
            opts: &self.opts,
        }
    }
}

pin_project! {
    #[project = CorsFutureStateProj]
    enum CorsFutureState<'a, C: 'a, T, B>
    where
        T: Work<C, Request<B>>,
    {
        Init {
            work: &'a T,
            context: &'a C,
            req: Option<Request<B>>,
        },
        Future {
            passthrough: bool,
            preflight: bool,
            #[pin]
            future: T::Future<'a>,
        },
    }
}

pin_project! {
    pub struct CorsFuture<'a, C, T, B>
    where
        T: Work<C, Request<B>>,
    {
        #[pin]
        state: CorsFutureState<'a, C, T, B>,
        opts: &'a CorsOptions,
    }
}

impl<'a, C, T, B> Future for CorsFuture<'a, C, T, B>
where
    T: Work<C, Request<B>>,
    T::Error: Into<BoxError>,
    T::Output: IntoResponse<B>,
    <T::Output as IntoResponse<B>>::Error: Into<BoxError>,
{
    type Output = Result<Response<B>, Error>;

    fn poll(
        mut self: alloc::pin::Pin<&mut Self>,
        cx: &mut alloc::task::Context<'_>,
    ) -> alloc::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                CorsFutureStateProj::Init { work, req, context } => {
                    let req = req.take().expect("Request");
                    let preflight = req.method() == Method::OPTIONS;

                    if preflight {
                        let passthrough =
                            req.headers().get(ACCESS_CONTROL_REQUEST_METHOD).is_none();
                        let future = work.call(*context, req);

                        this.state.set(CorsFutureState::Future {
                            passthrough,
                            preflight,
                            future,
                        });
                    } else {
                        let future = work.call(*context, req);
                        this.state.set(CorsFutureState::Future {
                            passthrough: false,
                            preflight,
                            future,
                        });
                    }
                }
                CorsFutureStateProj::Future {
                    passthrough,
                    preflight,
                    future,
                } => {
                    let resp = match ready!(future.poll(cx)) {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(Error::new(err))),
                    };

                    let mut resp = match resp.into_response() {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(Error::new(err))),
                    };

                    if *passthrough {
                        return Poll::Ready(Ok(resp));
                    }

                    this.opts.apply(resp.headers_mut(), *preflight);

                    return Poll::Ready(Ok(resp));
                }
            }
        }
    }
}
