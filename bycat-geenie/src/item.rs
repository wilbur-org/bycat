use relative_path::RelativePathBuf;
use spurgt::Spurgt;

use crate::{Context, GeenieError, result::ResultBuilder};
use core::{future::Future, pin::Pin};

pub trait Item<C, B> {
    fn process<'a>(
        self,
        ctx: Context<'a, C, B>,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a;
}

impl<T, C, B> Item<C, B> for T
where
    T: 'static,
    for<'a> T: FnOnce(Context<'a, C, B>) -> Result<(), GeenieError>,
{
    fn process<'a>(
        self,
        ctx: Context<'a, C, B>,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a {
        async move { (self)(ctx) }
    }
}

pub trait ItemExt<C, B>: Item<C, B> {
    fn mount<P>(self, path: P) -> MountItem<Self>
    where
        Self: Sized,
        P: Into<RelativePathBuf>,
    {
        MountItem {
            item: self,
            mount: path.into(),
        }
    }
}

impl<T, C, B> ItemExt<C, B> for T where T: Item<C, B> {}

pub trait DynamicItem<C, B> {
    fn process<'a>(
        self: Box<Self>,
        ctx: Context<'a, C, B>,
    ) -> Pin<Box<dyn Future<Output = Result<(), GeenieError>> + 'a>>;
}

pub struct ItemBox<T>(pub T);

impl<T, C, B> DynamicItem<C, B> for ItemBox<T>
where
    T: Item<C, B> + 'static,
{
    fn process<'a>(
        self: Box<Self>,
        ctx: Context<'a, C, B>,
    ) -> Pin<Box<dyn Future<Output = Result<(), GeenieError>> + 'a>> {
        Box::pin(async move { self.0.process(ctx).await })
    }
}

impl<C, B> Item<C, B> for ItemBox<Box<dyn DynamicItem<C, B>>> {
    fn process<'a>(
        self,
        ctx: Context<'a, C, B>,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a {
        async move { self.0.process(ctx).await }
    }
}

pub struct MountItem<T> {
    item: T,
    mount: RelativePathBuf,
}

impl<T> MountItem<T> {
    pub fn new(mount: impl Into<RelativePathBuf>, item: T) -> MountItem<T> {
        MountItem {
            item,
            mount: mount.into(),
        }
    }
}

impl<T, C, B> Item<C, B> for MountItem<T>
where
    C: 'static,
    T: Item<C, B> + 'static,
    B: 'static,
{
    fn process<'a>(
        self,
        mut ctx: Context<'a, C, B>,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            let mut files = ResultBuilder::default();
            let mut items = Vec::default();

            self.item
                .process(Context {
                    files: &mut files,
                    questions: &mut items,
                    ctx: ctx.ctx,
                })
                .await?;

            for file in files.files {
                ctx.package(file.map_path(|path| self.mount.join(path)))?;
            }

            for item in items {
                ctx.push(MountItem {
                    item: ItemBox(item),
                    mount: self.mount.clone(),
                });
            }

            Ok(())
        }
    }
}
