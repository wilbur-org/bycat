use crate::{IntoResponse, body::HttpBody};
use alloc::{convert::Infallible, fmt};
use bycat_error::{BoxError, Error};
use bytes::Bytes;
use http::{Response, StatusCode};

#[derive(Debug)]
enum ErrorKind {
    NotFound,
    MaxSizeReached,
    Internal(Error),
}

#[derive(Debug)]
pub struct HttpError {
    kind: ErrorKind,
}

impl HttpError {
    pub fn not_found() -> HttpError {
        HttpError {
            kind: ErrorKind::NotFound,
        }
    }

    pub fn max_size_reached() -> HttpError {
        HttpError {
            kind: ErrorKind::MaxSizeReached,
        }
    }

    pub fn custom<T: Into<BoxError>>(custom: T) -> HttpError {
        HttpError {
            kind: ErrorKind::Internal(Error::new(custom)),
        }
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::NotFound => {
                write!(f, "Not Found")
            }
            ErrorKind::MaxSizeReached => {
                write!(f, "Maximum Size Reached")
            }
            ErrorKind::Internal(error) => {
                write!(f, "{error}")
            }
        }
    }
}

impl core::error::Error for HttpError {
    fn source(&self) -> Option<&(dyn alloc::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::NotFound => None,
            ErrorKind::MaxSizeReached => None,
            ErrorKind::Internal(error) => Some(&*error),
        }
    }
}

impl<B: HttpBody> IntoResponse<B> for HttpError {
    type Error = Infallible;

    fn into_response(self) -> Result<http::Response<B>, Self::Error> {
        let (body, status) = match &self.kind {
            ErrorKind::NotFound => (
                B::from_bytes(Bytes::from("Not Found")),
                StatusCode::NOT_FOUND,
            ),
            ErrorKind::MaxSizeReached => (
                B::from_bytes(Bytes::from("Maximum Size Reached")),
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            ErrorKind::Internal(_) => (
                B::from_bytes(Bytes::from("Internal Server Error")),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };

        let mut resp = Response::new(body);
        *resp.status_mut() = status;

        Ok(resp)
    }
}

impl From<Error> for HttpError {
    fn from(value: Error) -> Self {
        HttpError {
            kind: ErrorKind::Internal(value),
        }
    }
}

impl From<Infallible> for HttpError {
    fn from(value: Infallible) -> Self {
        HttpError {
            kind: ErrorKind::Internal(value.into()),
        }
    }
}
