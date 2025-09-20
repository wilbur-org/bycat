use std::{marker::PhantomData, sync::Arc};

use bycat::Work;
use bycat_service::Shutdown;
use directories::{BaseDirs, ProjectDirs};
use pin_project_lite::pin_project;

use crate::{paths::Paths, req::CliRequest, settings::Settings};

pub struct ConfigBuilder<'a> {
    local: Option<&'a str>,
    pattern: Option<&'a str>,
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

    pub fn build<C, T>(self, work: T) -> Cli<C, T> {
        let base = BaseDirs::new().unwrap();
        let project = ProjectDirs::from("", "", &self.name).unwrap();

        Cli {
            work,
            paths: (base, project).into(),
            ctx: PhantomData,
        }
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
{
    pub async fn run_with(&self, req: CliRequest) -> Result<T::Output, T::Error> {
        self.call(&(), req).await
    }

    pub async fn run(&self) -> Result<T::Output, T::Error> {
        self.run_with(CliRequest::from_env()).await
    }
}

impl<C, T> Work<C, CliRequest> for Cli<C, T>
where
    T: Work<C, App>,
{
    type Error = T::Error;
    type Output = T::Output;

    type Future<'a>
        = CliWorkFuture<'a, C, T>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, ctx: &'a C, req: CliRequest) -> Self::Future<'a> {
        CliWorkFuture {
            task: &self.work,
            ctx,
            state: CliWorkState::Init {
                paths: Some(self.paths.clone()),
            },
            req,
        }
    }
}

pin_project! {

#[project = CliProj]
enum CliWorkState<T> {
    Init{
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
    #[pin]
    state: CliWorkState<T::Future<'a>>,
    req: CliRequest
}
}

impl<'a, C, T> Future for CliWorkFuture<'a, C, T>
where
    T: Work<C, App>,
{
    type Output = Result<T::Output, T::Error>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                CliProj::Init { paths } => {
                    let paths = paths.take().unwrap();

                    let app = App(Arc::new(AppInner {
                        paths,
                        settings: Settings::default(),
                        args: this.req.args.clone(),
                        shutdown: Shutdown::new(),
                    }));

                    let future = this.task.call(*&this.ctx, app);

                    this.state.set(CliWorkState::Future { future });
                }
                CliProj::Future { future } => return future.poll(cx),
            }
        }
    }
}

struct AppInner {
    paths: Paths,
    settings: Settings,
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

    pub fn settings(&self) -> &Settings {
        &self.0.settings
    }
}
