use alloc::format;
use bycat_container::Extensible;
use bycat_error::Error;
use core::any::{Any, type_name};
use core::future::{self, Ready};

use crate::FromRequestParts;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Ext<T>(pub T);

impl<T, C> FromRequestParts<C> for Ext<T>
where
    C: Extensible,
    T: Clone + Send + Sync + Any,
{
    type Future<'a>
        = Ready<Result<Self, Error>>
    where
        C: 'a;

    fn from_request_parts<'a>(
        parts: &'a mut http::request::Parts,
        _state: &'a C,
    ) -> Self::Future<'a> {
        future::ready(
            parts
                .extensions
                .get::<T>()
                .cloned()
                .map(Ext)
                .ok_or_else(|| {
                    Error::new(format!("Type {:?} not found in state", type_name::<T>()))
                }),
        )
    }
}
