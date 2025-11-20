use bycat::{Middleware, Work};
use bycat_error::Error;
use heather::{HBoxFuture, HSend, HSendSync, Hrc};
use http::{Request, Response};
use std::future::Future;

use crate::IntoResponse;

pub trait Modifier<B, C>: HSendSync {
    type Modify: Modify<B, C>;
    fn before<'a>(
        &'a self,
        request: &'a mut Request<B>,
        state: &'a C,
    ) -> impl Future<Output = Self::Modify> + 'a + HSend;
}

pub trait Modify<B, C>: HSend {
    fn modify<'a>(
        self,
        response: &'a mut Response<B>,
        state: &'a C,
    ) -> impl Future<Output = ()> + 'a + HSend;
}

pub trait DynModifier<B, C>: HSendSync {
    fn before<'a>(
        &'a self,
        request: &'a mut Request<B>,
        state: &'a C,
    ) -> HBoxFuture<'a, BoxModify<B, C>>;
}

pub trait DynModify<B, C>: HSend {
    fn modify<'a>(
        self: Box<Self>,
        response: &'a mut Response<B>,
        state: &'a C,
    ) -> HBoxFuture<'a, ()>;
}

struct ModifyBox<T>(T);

impl<T, B: HSend, C: HSendSync> DynModify<B, C> for ModifyBox<T>
where
    T: Modify<B, C> + HSend + 'static,
{
    fn modify<'a>(
        self: Box<Self>,
        response: &'a mut Response<B>,
        state: &'a C,
    ) -> HBoxFuture<'a, ()> {
        Box::pin(async move { self.0.modify(response, state).await })
    }
}

#[derive(Debug, Clone, Copy)]
struct ModifierBox<T>(T);

impl<T, B, C> DynModifier<B, C> for ModifierBox<T>
where
    T: Modifier<B, C> + HSendSync,
    T::Modify: HSend + 'static,
    C: HSendSync,
    B: HSend,
{
    fn before<'a>(
        &'a self,
        request: &'a mut Request<B>,
        state: &'a C,
    ) -> HBoxFuture<'a, BoxModify<B, C>> {
        Box::pin(async move {
            Box::new(ModifyBox(self.0.before(request, state).await)) as BoxModify<B, C>
        })
    }
}

pub fn modifier_box<'a, T, B, C>(modifier: T) -> BoxModifier<'a, B, C>
where
    T: Modifier<B, C> + HSendSync + 'a,
    T::Modify: HSend + 'static,
    C: HSendSync + 'a,
    B: HSend,
{
    Hrc::new(ModifierBox(modifier))
}

pub type BoxModifier<'a, B, C> = Hrc<dyn DynModifier<B, C> + 'a>;
pub type BoxModify<B, C> = Box<dyn DynModify<B, C>>;

pub struct ModifierMiddleware<'a, B, C> {
    modifiers: Hrc<[BoxModifier<'a, B, C>]>,
}

impl<'a, B, C> ModifierMiddleware<'a, B, C> {
    pub fn new(modifiers: impl Into<Hrc<[BoxModifier<'a, B, C>]>>) -> ModifierMiddleware<'a, B, C> {
        ModifierMiddleware {
            modifiers: modifiers.into(),
        }
    }
}

impl<'a, B, C, H> Middleware<C, Request<B>, H> for ModifierMiddleware<'a, B, C>
where
    B: HSend,
    C: HSendSync,
    H: Work<C, Request<B>> + HSendSync,
    H::Output: IntoResponse<B>,
    H::Error: Into<Error>,
{
    type Work = ModifierMiddlewareHandler<'a, B, C, H>;

    fn wrap(&self, handler: H) -> Self::Work {
        ModifierMiddlewareHandler {
            modifiers: self.modifiers.clone(),
            handler,
        }
    }
}

pub struct ModifierMiddlewareHandler<'a, B, C, H> {
    modifiers: Hrc<[BoxModifier<'a, B, C>]>,
    handler: H,
}

impl<'b, B, C, H> Work<C, Request<B>> for ModifierMiddlewareHandler<'b, B, C, H>
where
    B: HSend,
    C: HSendSync,
    H: Work<C, Request<B>> + HSendSync,
    H::Output: IntoResponse<B>,
    H::Error: Into<Error>,
{
    type Output = Response<B>;
    type Error = Error;
    type Future<'a>
        = HBoxFuture<'a, Result<Self::Output, Error>>
    where
        Self: 'a,
        C: 'a,
        B: 'a,
        H: 'a;

    fn call<'a>(&'a self, context: &'a C, mut req: Request<B>) -> Self::Future<'a> {
        let modifiers = self.modifiers.clone();

        Box::pin(async move {
            let mut mods = Vec::with_capacity(modifiers.len());

            for modifier in modifiers.iter() {
                mods.push(modifier.before(&mut req, context).await);
            }

            let mut res = self
                .handler
                .call(context, req)
                .await
                .map_err(Into::into)?
                .into_response();

            for modifier in mods {
                modifier.modify(&mut res, context).await;
            }

            Ok(res)
        })
    }
}

impl<'b, B: HSend, C: HSendSync> Modifier<B, C> for Hrc<[BoxModifier<'b, B, C>]> {
    type Modify = Vec<BoxModify<B, C>>;
    fn before<'a>(
        &'a self,
        request: &'a mut Request<B>,
        state: &'a C,
    ) -> impl Future<Output = Self::Modify> + 'a + HSend {
        async move {
            let mut modifiers = Vec::with_capacity(self.len());
            for m in self.iter() {
                modifiers.push(m.before(request, state).await);
            }

            modifiers
        }
    }
}

impl<B: HSend, C: HSendSync> Modify<B, C> for Vec<BoxModify<B, C>> {
    fn modify<'a>(
        self,
        response: &'a mut Response<B>,
        state: &'a C,
    ) -> impl Future<Output = ()> + 'a + HSend {
        async move {
            for m in self {
                m.modify(response, state).await;
            }
        }
    }
}

pub struct ModifierList<'b, B, C>(Hrc<[BoxModifier<'b, B, C>]>);

impl<'b, B, C> Clone for ModifierList<'b, B, C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<'a, B, C> From<Hrc<[BoxModifier<'a, B, C>]>> for ModifierList<'a, B, C> {
    fn from(value: Hrc<[BoxModifier<'a, B, C>]>) -> Self {
        ModifierList(value)
    }
}

impl<'a, B, C> From<Vec<BoxModifier<'a, B, C>>> for ModifierList<'a, B, C> {
    fn from(value: Vec<BoxModifier<'a, B, C>>) -> Self {
        ModifierList(value.into())
    }
}

impl<'a, B, C> From<ModifierList<'a, B, C>> for Vec<BoxModifier<'a, B, C>> {
    fn from(value: ModifierList<'a, B, C>) -> Self {
        value.0.iter().cloned().collect()
    }
}

impl<'b, B: HSend, C: HSendSync> Modifier<B, C> for ModifierList<'b, B, C> {
    type Modify = Vec<BoxModify<B, C>>;

    fn before<'a>(
        &'a self,
        request: &'a mut Request<B>,
        state: &'a C,
    ) -> impl Future<Output = Self::Modify> + 'a + HSend {
        self.0.before(request, state)
    }
}
