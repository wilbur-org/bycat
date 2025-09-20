use bycat::Work;
use directories::ProjectDirs;

use crate::req::CliRequest;

pub struct Builder<'a> {
    name: &'a str,
}

impl<'a> Builder<'a> {
    fn build<T>(self, work: T) -> Cli<T> {
        let Some(dirs) = ProjectDirs::from("", "", self.name) else {
            panic!("")
        };

        let app = App { dirs };
        Cli { app, work }
    }

    // pub async fn run<T>(self, work: T, req: CliRequest) -> Result<T::Output, T::Error>
    // where
    //     T: Work<App, CliRequest>,
    // {
    //     let app = self.build();
    //     work.call(&app, req).await
    // }
}

pub struct Cli<T> {
    app: App,
    work: T,
}

impl<T> Cli<T>
where
    T: Work<App, CliRequest>,
{
    pub async fn run_with(&self, req: CliRequest) -> Result<T::Output, T::Error> {
        self.work.call(&self.app, req).await
    }
}

impl<C, T> Work<C, CliRequest> for Cli<T>
where
    T: Work<App, CliRequest>,
{
    type Error = T::Error;
    type Output = T::Output;

    type Future<'a>
        = T::Future<'a>
    where
        Self: 'a,
        C: 'a;

    fn call<'a>(&'a self, _context: &'a C, req: CliRequest) -> Self::Future<'a> {
        self.work.call(&self.app, req)
    }
}

pub struct App {
    dirs: ProjectDirs,
}

impl App {}
