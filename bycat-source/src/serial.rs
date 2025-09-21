use alloc::{collections::VecDeque, vec::Vec};
use core::{
    marker::PhantomData,
    task::{ready, Poll},
};

use futures::Stream;
use pin_project_lite::pin_project;

use crate::Source;

#[derive(Debug, Clone)]
pub struct Serial<S> {
    sources: Vec<S>,
}

impl<S> Serial<S> {
    pub fn new(sources: Vec<S>) -> Serial<S> {
        Serial { sources }
    }

    pub fn push(&mut self, source: S) {
        self.sources.push(source);
    }
}

impl<C, S> Source<C> for Serial<S>
where
    S: Source<C>,
{
    type Error = S::Error;
    type Item = S::Item;
    type Stream<'a>
        = SerialStream<'a, C, S>
    where
        Self: 'a,
        C: 'a;

    fn create_stream<'a>(self, ctx: &'a C) -> Self::Stream<'a> {
        SerialStream {
            state: SerialState::Next,
            streams: self
                .sources
                .into_iter()
                .map(|m| m.create_stream(ctx))
                .collect(),
            ctx: PhantomData,
        }
    }
}

pin_project! {
    #[project = StateProj]
    enum SerialState<S> {
        Next,
        Stream {
            #[pin]
            stream: S
        },
        Done,
    }
}

pin_project! {

    pub struct SerialStream<'a, C: 'a, S: 'a>
    where
        S: Source<C>,
    {
        #[pin]
        state: SerialState<S::Stream<'a>>,
        ctx: PhantomData<C>,
        streams: VecDeque<S::Stream<'a>>
    }

}

impl<'a, C: 'a, S: 'a> Stream for SerialStream<'a, C, S>
where
    S: Source<C>,
{
    type Item = Result<S::Item, S::Error>;

    fn poll_next(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Option<Self::Item>> {
        loop {
            let mut this = self.as_mut().project();

            match this.state.as_mut().project() {
                StateProj::Next => {
                    let Some(stream) = this.streams.pop_front() else {
                        this.state.set(SerialState::Done);
                        continue;
                    };

                    this.state.set(SerialState::Stream { stream });
                }
                StateProj::Stream { stream } => match ready!(stream.poll_next(cx)) {
                    Some(ret) => return Poll::Ready(Some(ret)),
                    None => {
                        this.state.set(SerialState::Next);
                    }
                },
                StateProj::Done => return Poll::Ready(None),
            }
        }
    }
}
