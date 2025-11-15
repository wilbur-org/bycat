use std::{
    path::PathBuf,
    task::{Poll, ready},
};

use bycat_error::Error;
use bycat_fs::{
    FileResolver,
    fs::{IntoResolverListStream, IntoResolverWalkStream, ResolvedPath},
};
use bycat_package::match_glob;
use futures::Stream;
use pin_project_lite::pin_project;
use tinytemplate::TinyTemplate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Single,
    Many,
    Recursive,
}

impl Mode {
    fn is_single(&self) -> bool {
        matches!(self, Mode::Single)
    }
}

#[derive(Debug, Clone)]
pub struct Locator {
    root: PathBuf,
    pattern: Option<String>,
    mode: Mode,
}

impl Locator {
    pub fn new(root: impl Into<PathBuf>) -> Locator {
        Locator {
            root: root.into(),
            pattern: None,
            mode: Mode::Many,
        }
    }

    pub fn pattern(mut self, pattern: impl ToString) -> Self {
        self.pattern = Some(pattern.to_string());
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }
}

#[derive(Debug, serde::Serialize)]
struct Context<'a> {
    ext: &'a str,
}

impl Locator {
    pub fn find(&self, exts: &[String]) -> bycat_error::Result<LocatorStream> {
        let mut template = TinyTemplate::new();

        template
            .add_template(
                "main",
                self.pattern.as_ref().map(|m| &**m).unwrap_or("*.{ext}"),
            )
            .map_err(Error::new)?;

        let stream = match &self.mode {
            Mode::Recursive => {
                let mut readdir = FileResolver::new(self.root.clone());

                for ext in exts {
                    let glob = template
                        .render("main", &Context { ext })
                        .map_err(Error::new)?;
                    readdir = readdir.pattern(match_glob(glob))
                }

                LocatorStream {
                    state: StreamState::Walk {
                        stream: readdir.into_walkdir(),
                    },
                }
            }
            Mode::Single | Mode::Many => {
                let mut readdir = FileResolver::new(self.root.clone());

                for ext in exts {
                    let glob = template
                        .render("main", &Context { ext })
                        .map_err(Error::new)?;
                    readdir = readdir.pattern(match_glob(glob))
                }

                LocatorStream {
                    state: StreamState::Read {
                        single: self.mode.is_single(),
                        stream: readdir.into_list_dir(),
                    },
                }
            }
        };

        Ok(stream)
    }
}

pin_project! {

    #[project = StateProj]
    enum StreamState {
        Read {
            single: bool,
            #[pin]
            stream: IntoResolverListStream,
        },
        Walk {
            #[pin]
            stream: IntoResolverWalkStream
        },
        Done
    }

}

pin_project! {

pub struct LocatorStream {
    #[pin]
    state: StreamState,
}
}

impl Stream for LocatorStream {
    type Item = bycat_error::Result<ResolvedPath>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                StateProj::Read { single, stream } => {
                    match ready!(stream.poll_next(cx)) {
                        Some(Ok(ret)) => {
                            //
                            if *single {
                                this.state.set(StreamState::Done);
                            }
                            return Poll::Ready(Some(Ok(ret)));
                        }
                        Some(Err(ret)) => {
                            return Poll::Ready(Some(Err(ret)));
                        }
                        None => {
                            this.state.set(StreamState::Done);
                        }
                    }
                }
                StateProj::Walk { stream } => match ready!(stream.poll_next(cx)) {
                    Some(ret) => {
                        return Poll::Ready(Some(ret.map_err(Error::new)));
                    }
                    None => {
                        this.state.set(StreamState::Done);
                    }
                },
                StateProj::Done => return Poll::Ready(None),
            }
        }
    }
}
