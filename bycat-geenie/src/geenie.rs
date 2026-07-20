use core::{future::Future, pin::Pin};

use crate::{
    Context, GeenieError, Item,
    file::PackageItem,
    item::{DynamicItem, ItemBox},
    result::{GeenieResult, ResultBuilder},
};
use bycat_package::IntoPackage;
use spurgt::{Spurgt, core::BoxError};

pub struct Geenie<C, B> {
    items: Vec<Box<dyn DynamicItem<C, B>>>,
}

impl<C, B> Geenie<C, B> {
    pub fn new() -> Geenie<C, B> {
        Geenie {
            items: Default::default(),
        }
    }

    // pub fn env(&mut self) -> &mut Spurgt<E> {
    //     &mut self.env
    // }

    pub fn push<T>(&mut self, item: T) -> &mut Self
    where
        T: Item<C, B> + 'static,
    {
        self.items.push(Box::new(ItemBox(item)));
        self
    }

    pub fn package<P>(&mut self, file: P) -> Result<&mut Self, GeenieError>
    where
        P: IntoPackage<B> + 'static,
        P::Error: Into<BoxError>,
    {
        self.push(PackageItem::new(file));
        Ok(self)
    }

    pub async fn run(mut self, context: &mut C) -> Result<GeenieResult<B>, GeenieError> {
        let mut files = ResultBuilder::<B>::default();
        for item in self.items {
            Self::process_item(item, &mut files, context).await?;
        }

        Ok(files.build())
    }

    fn process_item<'a>(
        item: Box<dyn DynamicItem<C, B>>,
        files: &'a mut ResultBuilder<B>,
        context: &'a mut C,
    ) -> Pin<Box<dyn Future<Output = Result<(), GeenieError>> + 'a>>
    where
        C: 'a,
    {
        Box::pin(async move {
            let mut questions = Vec::default();

            item.process(Context {
                files,
                questions: &mut questions,
                ctx: context,
            })
            .await?;

            for question in questions {
                Self::process_item(question, files, context).await?;
            }

            Ok(())
        })
    }
}

impl<C, B> Item<C, B> for Geenie<C, B> {
    fn process<'a>(
        self,
        ctx: Context<'a, C, B>,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            for item in self.items {
                item.process(Context {
                    files: ctx.files,
                    questions: ctx.questions,
                    ctx: ctx.ctx,
                })
                .await?;
            }

            Ok(())
        }
    }
}
