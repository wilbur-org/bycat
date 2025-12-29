use crate::request::Request;
use bycat::Work;
use bycat_error::Error;
use futures_core::future::LocalBoxFuture;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    rc::Rc,
    string::{String, ToString},
};

trait Command<'js, C> {
    fn call<'a>(&'a self, ctx: &'a C, req: Request) -> LocalBoxFuture<'a, Result<(), Error>>;
}

struct CommandImpl<T>(T);

impl<'js, T, C> Command<'js, C> for CommandImpl<T>
where
    T: Work<C, Request, Output = (), Error = Error>,
{
    fn call<'a>(&'a self, ctx: &'a C, req: Request) -> LocalBoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            self.0.call(ctx, req).await?;
            Ok(())
        })
    }
}

pub struct Router<'js, C> {
    entries: Rc<BTreeMap<String, Rc<dyn Command<'js, C> + 'js>>>,
}

impl<'js, C> Clone for Router<'js, C> {
    fn clone(&self) -> Self {
        Router {
            entries: self.entries.clone(),
        }
    }
}

impl<'js, C> Default for Router<'js, C> {
    fn default() -> Self {
        Router {
            entries: Default::default(),
        }
    }
}

impl<'js, C> Router<'js, C> {
    pub fn add_command<W>(&mut self, name: &str, cmd: W) -> &mut Self
    where
        W: Work<C, Request, Output = (), Error = Error> + 'js,
    {
        Rc::make_mut(&mut self.entries).insert(name.to_string(), Rc::new(CommandImpl(cmd)));
        self
    }

    pub fn merge(&mut self, router: Router<'js, C>) -> &mut Self {
        Rc::make_mut(&mut self.entries)
            .extend(router.entries.iter().map(|(k, v)| (k.clone(), v.clone())));
        self
    }
}

impl<'js, C> Work<C, Request> for Router<'js, C> {
    type Output = ();

    type Error = Error;

    type Future<'a>
        = LocalBoxFuture<'a, Result<(), Error>>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request) -> Self::Future<'a> {
        Box::pin(async move {
            let Some(first) = req.args().first() else {
                return Err(Error::new("No command"));
            };

            let Some(found) = self.entries.get(first) else {
                return Err(Error::new("Command not found"));
            };

            found.call(context, req).await
        })
    }
}
