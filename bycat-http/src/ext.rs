use bycat_service::{Matcher, Service};
use http::Request;

use crate::{
    Error, IntoResponse,
    matcher::{FilterService, FilteredService, Or},
};

pub trait HttpWorkExt<C, B>: Service<C, Request<B>> {
    fn with_filter<M>(self, matcher: M) -> FilterService<Self, M>
    where
        Self: Sized,
        M: Matcher<Request<B>>,
    {
        FilterService::new(self, matcher)
    }

    fn or<T2>(self, other: T2) -> Or<Self, T2, B>
    where
        Self: Sized,
        T2: FilteredService<C, B>,
        Self::Output: IntoResponse<B>,
        Self::Error: Into<Error>,
        T2::Output: IntoResponse<B>,
        T2::Error: Into<Error>,
    {
        Or::new(self, other)
    }
}

impl<T, C, B> HttpWorkExt<C, B> for T where T: Service<C, Request<B>> {}
