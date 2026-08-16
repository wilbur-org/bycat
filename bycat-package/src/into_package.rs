use crate::{IntoPackage, Package};
use bycat_service::Service;

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

impl<C, B, R> Service<C, R> for IntoPackageWork<C, B>
where
    R: IntoPackage<B>,
{
    type Output = Package<B>;
    type Error = R::Error;

    type Future<'a>
        = R::Future
    where
        Self: 'a;

    fn call<'this: 'lifetime, 'ctx: 'lifetime, 'lifetime>(
        &'this self,
        _ctx: &'ctx C,
        package: R,
    ) -> Self::Future<'lifetime> {
        package.into_package()
    }
}
