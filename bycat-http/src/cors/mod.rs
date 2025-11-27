use alloc::{marker::PhantomData, sync::Arc};
use bycat::Work;
use bycat_error::{BoxError, Error};
use futures::future::{BoxFuture, LocalBoxFuture};
use http::{
    HeaderMap, HeaderValue, Method, Request, Response,
    header::{
        ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_ORIGIN,
        ACCESS_CONTROL_REQUEST_METHOD,
    },
};
use routing::router::MethodFilter;

use crate::{IntoResponse, body::HttpBody};

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

pub enum AllowedHeaders {
    Any,
    List(Vec<String>),
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
        }
    }
}

pub struct CorsOptions {
    origins: AllowedOrigins,
    headers: AllowedHeaders,
    methods: MethodFilter,
    max_age: u64,
    credentials: bool,
}

impl CorsOptions {
    fn apply(&self, headers: &mut HeaderMap) {
        self.origins.apply(headers);
        self.headers.apply(headers);
        if self.credentials {
            headers.insert(
                ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            );
        }

        if let Ok(value) = HeaderValue::from_str(&self.max_age.to_string()) {
            headers.insert(http::header::ACCESS_CONTROL_MAX_AGE, value);
        }
    }
}

pub struct Cors {
    opts: Arc<CorsOptions>,
}

impl Cors {}

pub struct CorsWork<T> {
    opts: Arc<CorsOptions>,
    work: T,
}

impl<T, C, B> Work<C, Request<B>> for CorsWork<T>
where
    T: Work<C, Request<B>>,
    T::Error: Into<BoxError>,
    T::Output: IntoResponse<B>,
    <T::Output as IntoResponse<B>>::Error: Into<BoxError>,
    B: 'static,
{
    type Output = Response<B>;

    type Error = Error;

    type Future<'a>
        = LocalBoxFuture<'a, Result<Self::Output, Self::Error>>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request<B>) -> Self::Future<'a> {
        Box::pin(async move {
            //

            let preflight = req.method() == Method::OPTIONS;

            if preflight {
                if req.headers().get(ACCESS_CONTROL_REQUEST_METHOD).is_none() {
                    todo!()
                }
            }

            let mut resp = self
                .work
                .call(context, req)
                .await
                .map_err(Error::new)?
                .into_response()
                .map_err(Error::new)?;

            self.opts.apply(resp.headers_mut());

            Ok(resp)
        })
    }
}
