#![no_std]

extern crate alloc;

mod builder;
mod service;
mod shutdown;

pub use self::{builder::*, service::*, shutdown::*};

pub trait ServiceFactory<C> {
    type Error;
    type Service;
    type Options;
    type Future<'b>: Future<Output = Result<Self::Service, Self::Error>>
    where
        Self: 'b,
        C: 'b;

    fn create<'b>(&'b self, ctx: C, options: Self::Options) -> Self::Future<'b>;
}
