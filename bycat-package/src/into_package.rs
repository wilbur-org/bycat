use crate::{IntoPackage, Package};
use bycat::Work;

use core::marker::PhantomData;

#[derive(Debug)]
pub struct IntoPackageWork<C, B> {
    pub(crate) ctx: PhantomData<fn() -> (C, B)>,
}

impl<C, B> IntoPackageWork<C, B> {
    pub fn new() -> IntoPackageWork<C, B> {
        IntoPackageWork { ctx: PhantomData }
    }
}

impl<C, B> Copy for IntoPackageWork<C, B> {}

impl<C, B> Clone for IntoPackageWork<C, B> {
    fn clone(&self) -> Self {
        IntoPackageWork { ctx: PhantomData }
    }
}

unsafe impl<C, B> Send for IntoPackageWork<C, B> {}

unsafe impl<C, B> Sync for IntoPackageWork<C, B> {}

impl<C, B, R> Work<C, R> for IntoPackageWork<C, B>
where
    R: IntoPackage<B>,
{
    type Output = Package<B>;
    type Error = R::Error;

    type Future<'a>
        = R::Future
    where
        Self: 'a;

    fn call<'a>(&'a self, _ctx: &'a C, package: R) -> Self::Future<'a> {
        package.into_package()
    }
}
