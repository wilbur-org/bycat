// TODO: Fix this
extern crate std as alloc;

mod ext;

#[cfg(feature = "std")]
pub mod body;

pub mod handler;
mod into_response;
#[cfg(feature = "router")]
pub mod router;

pub mod extract;

#[cfg(feature = "cookies")]
pub mod cookies;
pub mod cors;
#[cfg(feature = "multipart")]
pub mod multipart;
#[cfg(feature = "serve")]
pub mod serve;

#[cfg(feature = "session")]
pub mod session;

#[cfg(feature = "statics")]
mod statics;

#[cfg(feature = "serve-tokio")]
pub use self::serve::serve;

pub use self::{
    extract::{from_request::FromRequest, from_request_parts::FromRequestParts},
    handler::handler,
    into_response::*,
};

pub use http::{
    self, HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode, Uri, header,
};
