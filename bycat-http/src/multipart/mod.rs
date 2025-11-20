use bycat_error::{BoxError, Error};
use bytes::Bytes;
use http::header::CONTENT_TYPE;
pub use multer::{Field, Multipart};

use crate::FromRequest;

impl<'ctx, C, B> FromRequest<C, B> for Multipart<'ctx>
where
    B: http_body::Body<Data = Bytes> + Send + 'ctx,
    B::Error: Into<BoxError>,
{
    type Future<'a>
        = core::future::Ready<Result<Self, Error>>
    where
        C: 'a;

    fn from_request<'a>(req: http::Request<B>, _state: &'a C) -> Self::Future<'a> {
        let Some(boundary) = req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| multer::parse_boundary(ct).ok())
        else {
            return core::future::ready(Err(Error::new("Bad request")));
        };

        let stream = http_body_util::BodyDataStream::new(req.into_body());
        let multipart = Multipart::new(stream, boundary);

        core::future::ready(Ok(multipart))
    }
}
