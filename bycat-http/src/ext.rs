use bycat::{Matcher, Work};
use http::Request;

use crate::{
    Error, IntoResponse,
    matcher::{FilterWork, FilteredWork, Or},
};

pub trait HttpWorkExt<C, B>: Work<C, Request<B>> {
    fn with_filter<M>(self, matcher: M) -> FilterWork<Self, M>
    where
        Self: Sized,
        M: Matcher<Request<B>>,
    {
        FilterWork::new(self, matcher)
    }

    fn or<T2>(self, other: T2) -> Or<Self, T2>
    where
        Self: Sized,
        T2: FilteredWork<C, B>,
        Self::Output: IntoResponse<B>,
        Self::Error: Into<Error>,
        T2::Output: IntoResponse<B>,
        T2::Error: Into<Error>,
    {
        Or(self, other)
    }
}

impl<T, C, B> HttpWorkExt<C, B> for T where T: Work<C, Request<B>> {}
