use core::{future::Future, pin::Pin};

use crate::{
    command::{Command, CommandItem},
    item::{DynamicItem, ItemBox},
    result::{GeenieResult, ResultBuilder},
    Context, File, GeenieError, Item,
};
use spurgt::Spurgt;

pub struct Geenie<E, C> {
    env: Spurgt<E>,
    items: Vec<Box<dyn DynamicItem<E, C>>>,
}

impl<E, C> Default for Geenie<E, C>
where
    E: Default,
{
    fn default() -> Self {
        Geenie {
            env: Spurgt::default(),
            items: Default::default(),
        }
    }
}

impl<E: spurgt::core::Env, C> Geenie<E, C> {
    pub fn new(env: E) -> Geenie<E, C> {
        Geenie {
            env: Spurgt::new(env),
            items: Default::default(),
        }
    }

    pub fn env(&mut self) -> &mut Spurgt<E> {
        &mut self.env
    }

    pub fn push<T>(&mut self, item: T) -> &mut Self
    where
        T: Item<E, C> + 'static,
    {
        self.items.push(Box::new(ItemBox(item)));
        self
    }

    pub fn command<T>(&mut self, command: T) -> &mut Self
    where
        T: Command<E> + 'static,
    {
        self.push(CommandItem(command));
        self
    }

    pub fn file(&mut self, file: impl Into<File>) -> Result<&mut Self, GeenieError> {
        self.items.push(Box::new(ItemBox(file.into())));
        Ok(self)
    }

    pub async fn run(mut self, context: &mut C) -> Result<GeenieResult<E>, GeenieError> {
        let mut files = ResultBuilder::<E>::default();
        for item in self.items {
            Self::process_item(&mut self.env, item, &mut files, context).await?;
        }

        Ok(files.build(self.env))
    }

    fn process_item<'a>(
        env: &'a mut Spurgt<E>,
        item: Box<dyn DynamicItem<E, C>>,
        files: &'a mut ResultBuilder<E>,
        context: &'a mut C,
    ) -> Pin<Box<dyn Future<Output = Result<(), GeenieError>> + 'a>>
    where
        C: 'a,
    {
        Box::pin(async move {
            let mut questions = Vec::default();

            item.process(
                Context {
                    files,
                    questions: &mut questions,
                    ctx: context,
                },
                env,
            )
            .await?;

            for question in questions {
                Self::process_item(env, question, files, context).await?;
            }

            Ok(())
        })
    }
}

impl<E, C> Item<E, C> for Geenie<E, C> {
    fn process<'a>(
        self,
        ctx: Context<'a, E, C>,
        env: &'a mut Spurgt<E>,
    ) -> impl Future<Output = Result<(), GeenieError>> + 'a {
        async move {
            for item in self.items {
                item.process(
                    Context {
                        files: ctx.files,
                        questions: ctx.questions,
                        ctx: ctx.ctx,
                    },
                    env,
                )
                .await?;
            }

            Ok(())
        }
    }
}
