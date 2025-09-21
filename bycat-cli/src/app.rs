use std::{
    marker::PhantomData,
    sync::Arc,
    task::{Poll, ready},
};

use bycat::Work;
use bycat_config::{Config, ConfigFactory, Locator, Mode};
use bycat_error::Error;
use bycat_service::Shutdown;
use directories::{BaseDirs, ProjectDirs};
use pin_project_lite::pin_project;

use crate::{paths::Paths, req::CliRequest};

pub struct ConfigBuilder<'a> {
    pub local: Option<&'a str>,
    pub pattern: Option<&'a str>,
}

pub struct Builder<'a> {
    name: &'a str,
    config: ConfigBuilder<'a>,
}

impl<'a> Builder<'a> {
    pub fn new(name: &'a str) -> Builder<'a> {
        Builder {
            name,
            config: ConfigBuilder {
                local: Default::default(),
                pattern: Default::default(),
            },
        }
    }

    pub fn build<C, T>(self, work: T) -> Result<Cli<C, T>, Error> {
        let base = BaseDirs::new().ok_or_else(|| Error::new("Could not acquire home directory"))?;
        let project = ProjectDirs::from("", "", &self.name)
            .ok_or_else(|| Error::new("Could not acquire home directory"))?;

        Ok(Cli {
            work,
            paths: (base, project).into(),
            ctx: PhantomData,
        })
    }
}

pub struct Cli<C, T> {
    paths: Paths,
    work: T,
    ctx: PhantomData<C>,
}

impl<T> Cli<(), T>
where
    T: Work<(), App>,
    T::Error: Into<Error>,
{
    pub async fn run_with(&self, req: CliRequest) -> Result<T::Output, Error> {
        self.call(&(), req).await
    }

    pub async fn run(&self) -> Result<T::Output, Error> {
        self.run_with(CliRequest::from_env()).await
    }
}

impl<C, T> Work<C, CliRequest> for Cli<C, T>
where
    T: Work<C, App>,
    T::Error: Into<Error>,
{
    type Error = Error;
    type Output = T::Output;

    type Future<'a>
        = CliWorkFuture<'a, C, T>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, ctx: &'a C, req: CliRequest) -> Self::Future<'a> {
        let mut factory = ConfigFactory::default();
        factory.add_locator(Locator::new(self.paths.config().path));
        factory.add_locator(
            Locator::new(req.cwd.clone())
                .mode(Mode::Many)
                .pattern("**/*.{ext}"),
        );

        CliWorkFuture {
            task: &self.work,
            ctx,
            state: CliWorkState::Init {
                paths: Some(self.paths.clone()),
            },
            req,
            factory,
        }
    }
}

pin_project! {

#[project = CliProj]
enum CliWorkState<T> {
    Init{
        paths: Option<Paths>
    },
    Settings {
        #[pin]
        future: bycat_config::LoadConfigFuture,
        paths: Option<Paths>
    },
    Future { #[pin]future: T },
}

}

pin_project! {
    pub struct CliWorkFuture<'a, C, T>
where
    T: Work<C, App>,
{
    task: &'a T,
    ctx: &'a C,
    factory: ConfigFactory,
    #[pin]
    state: CliWorkState<T::Future<'a>>,
    req: CliRequest
}
}

impl<'a, C, T> Future for CliWorkFuture<'a, C, T>
where
    T: Work<C, App>,
    T::Error: Into<Error>,
{
    type Output = Result<T::Output, Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                CliProj::Init { paths } => {
                    let paths = paths.take().unwrap();

                    let future = match this.factory.load_config() {
                        Ok(ret) => ret,
                        Err(err) => return Poll::Ready(Err(err)),
                    };

                    this.state.set(CliWorkState::Settings {
                        future,
                        paths: Some(paths),
                    });
                }
                CliProj::Settings { future, paths } => {
                    let settings = ready!(future.poll(cx));

                    let paths = paths.take().unwrap();

                    let app = App(Arc::new(AppInner {
                        paths,
                        settings,
                        args: this.req.args.clone(),
                        shutdown: Shutdown::new(),
                    }));

                    let future = this.task.call(*&this.ctx, app.clone());

                    this.state.set(CliWorkState::Future { future });
                }
                CliProj::Future { future } => return future.poll(cx).map_err(Into::into),
            }
        }
    }
}

struct AppInner {
    paths: Paths,
    settings: Config,
    args: Vec<String>,
    shutdown: Shutdown,
}

#[derive(Clone)]
pub struct App(Arc<AppInner>);

impl App {
    pub fn paths(&self) -> &Paths {
        &self.0.paths
    }

    pub fn args(&self) -> &[String] {
        &self.0.args
    }

    pub fn settings(&self) -> &Config {
        &self.0.settings
    }
}
