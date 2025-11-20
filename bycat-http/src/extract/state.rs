use alloc::format;
use bycat_container::{Extensible, ReadableContainer};
use bycat_error::Error;
use core::any::{Any, type_name};
use core::future::{self, Ready};

use crate::FromRequestParts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct State<T>(pub T);

impl<T, C> FromRequestParts<C> for State<T>
where
    C: Extensible,
    T: Clone + Any,
{
    type Future<'a>
        = Ready<Result<Self, Error>>
    where
        C: 'a;

    fn from_request_parts<'a>(
        _parts: &'a mut http::request::Parts,
        state: &'a C,
    ) -> Self::Future<'a> {
        future::ready(
            state.get::<T>().cloned().map(State).ok_or_else(|| {
                Error::new(format!("Type {:?} not found in state", type_name::<T>()))
            }),
        )
    }
}
