use crate::{IntoResponse, body::HttpBody};
use alloc::{convert::Infallible, fmt};
use bytes::Bytes;
use http::{Error as HttpError, Response, StatusCode};

pub type BoxError = alloc::boxed::Box<dyn alloc::error::Error + Send + Sync + 'static>;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
enum ErrorKind {
    NotFound,
    MaxSizeReached,
    Internal(BoxError),
    Http(HttpError),
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub fn not_found() -> Error {
        Error {
            kind: ErrorKind::NotFound,
        }
    }

    pub fn max_size_reached() -> Error {
        Error {
            kind: ErrorKind::MaxSizeReached,
        }
    }

    pub fn custom<T: Into<BoxError>>(custom: T) -> Error {
        Error {
            kind: ErrorKind::Internal(custom.into()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::NotFound => {
                write!(f, "Not Found")
            }
            ErrorKind::MaxSizeReached => {
                write!(f, "Maximum Size Reached")
            }
            ErrorKind::Http(err) => {
                write!(f, "HTTP Error: {err}")
            }
            ErrorKind::Internal(error) => {
                write!(f, "{error}")
            }
        }
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn alloc::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::NotFound => None,
            ErrorKind::MaxSizeReached => None,
            ErrorKind::Http(error) => Some(&*error),
            ErrorKind::Internal(error) => Some(&**error),
        }
    }
}

impl<B: HttpBody> IntoResponse<B> for Error {
    fn into_response(self) -> Response<B> {
        let (body, status) = match &self.kind {
            ErrorKind::NotFound => (
                B::from_bytes(Bytes::from("Not Found")),
                StatusCode::NOT_FOUND,
            ),
            ErrorKind::MaxSizeReached => (
                B::from_bytes(Bytes::from("Maximum Size Reached")),
                StatusCode::PAYLOAD_TOO_LARGE,
            ),
            ErrorKind::Http(http) => {
                let body = B::from_bytes(Bytes::from(format!("HTTP Error: {}", http)));
                (body, StatusCode::INTERNAL_SERVER_ERROR)
            }
            ErrorKind::Internal(_) => (
                B::from_bytes(Bytes::from("Internal Server Error")),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        };

        let mut resp = Response::new(body);
        *resp.status_mut() = status;

        resp
    }
}

impl From<HttpError> for Error {
    fn from(value: HttpError) -> Self {
        Error {
            kind: ErrorKind::Http(value),
        }
    }
}

impl From<Infallible> for Error {
    fn from(value: Infallible) -> Self {
        Error {
            kind: ErrorKind::Internal(value.into()),
        }
    }
}
