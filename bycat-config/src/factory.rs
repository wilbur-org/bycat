use std::task::{Poll, ready};

use bycat_fs::Body;
use bycat_package::{Decode, IntoPackageWork};
use bycat_source::{Pipeline, PipelineStream, Serial, Source, SourceExt, StreamSource};
use futures::Stream;
use pin_project_lite::pin_project;
use toback::Toback;
use tracing::debug;

use crate::{
    config::Config,
    locator::{Locator, LocatorStream},
};

#[derive(Default)]
pub struct ConfigFactory {
    locators: Vec<Locator>,
}

impl ConfigFactory {
    pub fn add_locator(&mut self, locator: Locator) {
        self.locators.push(locator);
    }

    pub fn load_config(&self) -> bycat_error::Result<LoadConfigFuture> {
        let toback: Toback<Config> = Toback::new();

        let mut sources = Serial::new(Default::default());

        for locator in &self.locators {
            sources.push(bycat_source::stream(locator.find(toback.extensions())?));
        }

        let sources = sources
            .pipe(IntoPackageWork::<(), Body>::new())
            .pipe(bycat_package::Decode::<Config>::new());

        let stream = sources.create_stream(&());

        let future: LoadConfigFuture = LoadConfigFuture {
            output: Some(Config::default()),
            stream,
        };

        Ok(future)
    }
}

pin_project! {
    pub struct LoadConfigFuture {
        output: Option<Config>,
        #[pin]
        stream: PipelineStream<
            'static,
            Pipeline<Serial<StreamSource<LocatorStream>>, IntoPackageWork<(), Body>, ()>,
            Decode<Config>,
            (),
        >,
    }
}

impl Future for LoadConfigFuture {
    type Output = Config;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        loop {
            let this = self.as_mut().project();

            match ready!(this.stream.poll_next(cx)) {
                Some(Ok(ret)) => {
                    let (parts, content) = ret.into_parts();
                    debug!(path = ?parts.name, "File loaded");
                    this.output.as_mut().unwrap().extend(content);
                }
                Some(Err(err)) => {
                    tracing::error!(error = %err, "File failed to load");
                    // TODO: What todo about errors?
                    continue;
                }
                None => {
                    let config = this.output.take().unwrap();
                    return Poll::Ready(config);
                }
            }
        }
    }
}
