use std::{collections::HashMap, path::PathBuf};

use bycat::Work;
use futures_core::future::LocalBoxFuture;

use crate::env::Environ;

#[derive(Debug, Clone)]
pub struct Request {
    pub args: Vec<String>,
    pub env: Environ,
    pub cwd: PathBuf,
}

impl Request {
    pub fn from_env() -> std::io::Result<Request> {
        let cwd = std::env::current_dir()?;
        Ok(Request {
            args: std::env::args().collect(),
            env: Environ::from_env(),
            cwd,
        })
    }
}

trait Command<'js, C> {
    fn call<'a>(&'a self, ctx: &'a C, req: Request) -> LocalBoxFuture<'a, Result<(), ()>>;
}

struct CommandImpl<T>(T);

impl<'js, T, C> Command<'js, C> for CommandImpl<T>
where
    T: Work<C, Request, Output = (), Error = ()>,
{
    fn call<'a>(&'a self, ctx: &'a C, req: Request) -> LocalBoxFuture<'a, Result<(), ()>> {
        Box::pin(async move {
            self.0.call(ctx, req).await?;
            Ok(())
        })
    }
}

pub struct Router<'js, C> {
    entries: HashMap<String, Box<dyn Command<'js, C> + 'js>>,
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
        W: Work<C, Request, Output = (), Error = ()> + 'js,
    {
        self.entries
            .insert(name.to_string(), Box::new(CommandImpl(cmd)));
        self
    }
}

impl<'js, C> Work<C, Request> for Router<'js, C> {
    type Output = ();

    type Error = ();

    type Future<'a>
        = LocalBoxFuture<'a, Result<(), ()>>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, context: &'a C, req: Request) -> Self::Future<'a> {
        Box::pin(async move {
            let Some(first) = req.args.first() else {
                todo!()
            };

            let Some(found) = self.entries.get(first) else {
                todo!()
            };

            found.call(context, req).await
        })
    }
}
