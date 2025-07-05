use core::future::Future;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use pin_project_lite::pin_project;

pin_project! {
    #[project = StateProj]
    enum State<T>
    where
        T: Future,
    {
        Future {
            #[pin]
            future: T
        },
        Value {
            value: Option<T::Output>
        }
    }
}

pin_project! {
    pub struct BycatFuture<T1>
    where
        T1: Future,
    {
        #[pin]
        fut1: State<T1>,

}
}

impl<T1> BycatFuture<T1>
where
    T1: Future,
{
    pub fn new(fut1: T1) -> Self {
        Self {
            fut1: State::Future { future: fut1 },
        }
    }
}

impl<T1> Future for BycatFuture<T1>
where
    T1: Future,
{
    type Output = (T1::Output,);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        match ret1 {
            Some(ret1) => {
                let ret1 = ret1.take().unwrap();
                Poll::Ready((ret1,))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
    pub struct BycatFuture2<T1, T2>
    where
        T1: Future,
        T2: Future,
    {
        #[pin]
        fut1: State<T1>,
        #[pin]
        fut2: State<T2>,
}
}

impl<T1, T2> BycatFuture2<T1, T2>
where
    T1: Future,
    T2: Future,
{
    pub fn new(fut1: T1, fut2: T2) -> Self {
        Self {
            fut1: State::Future { future: fut1 },
            fut2: State::Future { future: fut2 },
        }
    }
}

impl<T1, T2> Future for BycatFuture2<T1, T2>
where
    T1: Future,
    T2: Future,
{
    type Output = (T1::Output, T2::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret2 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret2) {
            (Some(ret1), Some(ret2)) => {
                let ret1 = ret1.take().unwrap();
                let ret2 = ret2.take().unwrap();
                Poll::Ready((ret1, ret2))
            }
            _ => Poll::Pending,
        }
    }
}

// 3

pin_project! {
   pub struct BycatFuture3<T1, T2, T3>
    where
        T1: Future,
        T2: Future,
        T3: Future,
    {
        #[pin]
        fut1: State<BycatFuture2<T1, T2>>,
        #[pin]
        fut2: State<T3>,
}
}

impl<T1, T2, T3> BycatFuture3<T1, T2, T3>
where
    T1: Future,
    T2: Future,
    T3: Future,
{
    pub fn new(fut1: T1, fut2: T2, fut3: T3) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture2::new(fut1, fut2),
            },
            fut2: State::Future { future: fut3 },
        }
    }
}

impl<T1, T2, T3> Future for BycatFuture3<T1, T2, T3>
where
    T1: Future,
    T2: Future,
    T3: Future,
{
    type Output = (T1::Output, T2::Output, T3::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret3 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret3) {
            (Some(ret1), Some(ret3)) => {
                let (ret1, ret2) = ret1.take().unwrap();
                let ret3 = ret3.take().unwrap();
                Poll::Ready((ret1, ret2, ret3))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
  pub struct BycatFuture4<T1, T2, T3, T4>
    where
        T1: Future,
        T2: Future,
        T3: Future,
        T4: Future,
    {
        #[pin]
        fut1: State<BycatFuture3<T1, T2, T3>>,
        #[pin]
        fut2: State<T4>,
}
}

impl<T1, T2, T3, T4> BycatFuture4<T1, T2, T3, T4>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
{
    pub fn new(fut1: T1, fut2: T2, fut3: T3, fut4: T4) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture3::new(fut1, fut2, fut3),
            },
            fut2: State::Future { future: fut4 },
        }
    }
}

impl<T1, T2, T3, T4> Future for BycatFuture4<T1, T2, T3, T4>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
{
    type Output = (T1::Output, T2::Output, T3::Output, T4::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret4 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret4) {
            (Some(ret1), Some(ret4)) => {
                let (ret1, ret2, ret3) = ret1.take().unwrap();
                let ret4 = ret4.take().unwrap();
                Poll::Ready((ret1, ret2, ret3, ret4))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
    pub struct BycatFuture5<T1, T2, T3, T4, T5>
    where
        T1: Future,
        T2: Future,
        T3: Future,
        T4: Future,
        T5: Future,
    {
        #[pin]
        fut1: State<BycatFuture4<T1, T2, T3, T4>>,
        #[pin]
        fut2: State<T5>,
    }
}

impl<T1, T2, T3, T4, T5> BycatFuture5<T1, T2, T3, T4, T5>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
{
    pub fn new(fut1: T1, fut2: T2, fut3: T3, fut4: T4, fut5: T5) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture4::new(fut1, fut2, fut3, fut4),
            },
            fut2: State::Future { future: fut5 },
        }
    }
}

impl<T1, T2, T3, T4, T5> Future for BycatFuture5<T1, T2, T3, T4, T5>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
{
    type Output = (T1::Output, T2::Output, T3::Output, T4::Output, T5::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret5 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret5) {
            (Some(ret1), Some(ret5)) => {
                let (ret1, ret2, ret3, ret4) = ret1.take().unwrap();
                let ret5 = ret5.take().unwrap();
                Poll::Ready((ret1, ret2, ret3, ret4, ret5))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
    pub struct BycatFuture6<T1, T2, T3, T4, T5, T6>
    where
        T1: Future,
        T2: Future,
        T3: Future,
        T4: Future,
        T5: Future,
        T6: Future,
    {
        #[pin]
        fut1: State<BycatFuture5<T1, T2, T3, T4, T5>>,
        #[pin]
        fut2: State<T6>,
    }
}

impl<T1, T2, T3, T4, T5, T6> BycatFuture6<T1, T2, T3, T4, T5, T6>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
{
    pub fn new(fut1: T1, fut2: T2, fut3: T3, fut4: T4, fut5: T5, fut6: T6) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture5::new(fut1, fut2, fut3, fut4, fut5),
            },
            fut2: State::Future { future: fut6 },
        }
    }
}

impl<T1, T2, T3, T4, T5, T6> Future for BycatFuture6<T1, T2, T3, T4, T5, T6>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
    );

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret6 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret6) {
            (Some(ret1), Some(ret6)) => {
                let (ret1, ret2, ret3, ret4, ret5) = ret1.take().unwrap();
                let ret6 = ret6.take().unwrap();
                Poll::Ready((ret1, ret2, ret3, ret4, ret5, ret6))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
    pub struct BycatFuture7<T1, T2, T3, T4, T5, T6, T7>
    where
        T1: Future,
        T2: Future,
        T3: Future,
        T4: Future,
        T5: Future,
        T6: Future,
        T7: Future,
    {
        #[pin]
        fut1: State<BycatFuture6<T1, T2, T3, T4, T5, T6>>,
        #[pin]
        fut2: State<T7>,
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> BycatFuture7<T1, T2, T3, T4, T5, T6, T7>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
{
    pub fn new(fut1: T1, fut2: T2, fut3: T3, fut4: T4, fut5: T5, fut6: T6, fut7: T7) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture6::new(fut1, fut2, fut3, fut4, fut5, fut6),
            },
            fut2: State::Future { future: fut7 },
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> Future for BycatFuture7<T1, T2, T3, T4, T5, T6, T7>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
    );

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret7 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret7) {
            (Some(ret1), Some(ret7)) => {
                let (ret1, ret2, ret3, ret4, ret5, ret6) = ret1.take().unwrap();
                let ret7 = ret7.take().unwrap();
                Poll::Ready((ret1, ret2, ret3, ret4, ret5, ret6, ret7))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
    pub struct BycatFuture8<T1, T2, T3, T4, T5, T6, T7, T8>
    where
        T1: Future,
        T2: Future,
        T3: Future,
        T4: Future,
        T5: Future,
        T6: Future,
        T7: Future,
        T8: Future,
    {
        #[pin]
        fut1: State<BycatFuture7<T1, T2, T3, T4, T5, T6, T7>>,
        #[pin]
        fut2: State<T8>,
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> BycatFuture8<T1, T2, T3, T4, T5, T6, T7, T8>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
{
    pub fn new(
        fut1: T1,
        fut2: T2,
        fut3: T3,
        fut4: T4,
        fut5: T5,
        fut6: T6,
        fut7: T7,
        fut8: T8,
    ) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture7::new(fut1, fut2, fut3, fut4, fut5, fut6, fut7),
            },
            fut2: State::Future { future: fut8 },
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> Future for BycatFuture8<T1, T2, T3, T4, T5, T6, T7, T8>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
        T8::Output,
    );

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret8 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret8) {
            (Some(ret1), Some(ret8)) => {
                let (ret1, ret2, ret3, ret4, ret5, ret6, ret7) = ret1.take().unwrap();
                let ret8 = ret8.take().unwrap();
                Poll::Ready((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
  pub struct BycatFuture9<T1, T2, T3, T4, T5, T6, T7, T8, T9>
  where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
  {
    #[pin]
    fut1: State<BycatFuture8<T1, T2, T3, T4, T5, T6, T7, T8>>,
    #[pin]
    fut2: State<T9>,
  }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> BycatFuture9<T1, T2, T3, T4, T5, T6, T7, T8, T9>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
{
    pub fn new(
        fut1: T1,
        fut2: T2,
        fut3: T3,
        fut4: T4,
        fut5: T5,
        fut6: T6,
        fut7: T7,
        fut8: T8,
        fut9: T9,
    ) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture8::new(fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8),
            },
            fut2: State::Future { future: fut9 },
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> Future for BycatFuture9<T1, T2, T3, T4, T5, T6, T7, T8, T9>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
        T8::Output,
        T9::Output,
    );

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret9 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret9) {
            (Some(ret1), Some(ret9)) => {
                let (ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8) = ret1.take().unwrap();
                let ret9 = ret9.take().unwrap();
                Poll::Ready((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
  pub struct BycatFuture10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>
  where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
  {
    #[pin]
    fut1: State<BycatFuture9<T1, T2, T3, T4, T5, T6, T7, T8, T9>>,
    #[pin]
    fut2: State<T10>,
  }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> BycatFuture10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
{
    pub fn new(
        fut1: T1,
        fut2: T2,
        fut3: T3,
        fut4: T4,
        fut5: T5,
        fut6: T6,
        fut7: T7,
        fut8: T8,
        fut9: T9,
        fut10: T10,
    ) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture9::new(fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9),
            },
            fut2: State::Future { future: fut10 },
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> Future
    for BycatFuture10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
        T8::Output,
        T9::Output,
        T10::Output,
    );

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret10 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret10) {
            (Some(ret1), Some(ret10)) => {
                let (ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9) = ret1.take().unwrap();
                let ret10 = ret10.take().unwrap();
                Poll::Ready((ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
  pub struct BycatFuture11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>
  where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
    T11: Future,
  {
    #[pin]
    fut1: State<BycatFuture10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>>,
    #[pin]
    fut2: State<T11>,
  }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>
    BycatFuture11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
    T11: Future,
{
    pub fn new(
        fut1: T1,
        fut2: T2,
        fut3: T3,
        fut4: T4,
        fut5: T5,
        fut6: T6,
        fut7: T7,
        fut8: T8,
        fut9: T9,
        fut10: T10,
        fut11: T11,
    ) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture10::new(
                    fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9, fut10,
                ),
            },
            fut2: State::Future { future: fut11 },
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11> Future
    for BycatFuture11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
    T11: Future,
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
        T8::Output,
        T9::Output,
        T10::Output,
        T11::Output,
    );

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret11 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret11) {
            (Some(ret1), Some(ret11)) => {
                let (ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10) =
                    ret1.take().unwrap();
                let ret11 = ret11.take().unwrap();
                Poll::Ready((
                    ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11,
                ))
            }
            _ => Poll::Pending,
        }
    }
}

pin_project! {
  pub struct BycatFuture12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
  where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
    T11: Future,
    T12: Future,
  {
    #[pin]
    fut1: State<BycatFuture11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>>,
    #[pin]
    fut2: State<T12>,
  }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
    BycatFuture12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
    T11: Future,
    T12: Future,
{
    pub fn new(
        fut1: T1,
        fut2: T2,
        fut3: T3,
        fut4: T4,
        fut5: T5,
        fut6: T6,
        fut7: T7,
        fut8: T8,
        fut9: T9,
        fut10: T10,
        fut11: T11,
        fut12: T12,
    ) -> Self {
        Self {
            fut1: State::Future {
                future: BycatFuture11::new(
                    fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9, fut10, fut11,
                ),
            },
            fut2: State::Future { future: fut12 },
        }
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12> Future
    for BycatFuture12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
    T8: Future,
    T9: Future,
    T10: Future,
    T11: Future,
    T12: Future,
{
    type Output = (
        T1::Output,
        T2::Output,
        T3::Output,
        T4::Output,
        T5::Output,
        T6::Output,
        T7::Output,
        T8::Output,
        T9::Output,
        T10::Output,
        T11::Output,
        T12::Output,
    );

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();

        let ret1 = 'loop_fut1: loop {
            match this.fut1.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut1.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut1 None,
                },
                StateProj::Value { value } => break 'loop_fut1 Some(value),
            }
        };

        let ret12 = 'loop_fut2: loop {
            match this.fut2.as_mut().project() {
                StateProj::Future { future } => match future.poll(cx) {
                    Poll::Ready(ret) => {
                        this.fut2.set(State::Value { value: Some(ret) });
                    }
                    Poll::Pending => break 'loop_fut2 None,
                },
                StateProj::Value { value } => break 'loop_fut2 Some(value),
            }
        };

        match (ret1, ret12) {
            (Some(ret1), Some(ret12)) => {
                let (ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11) =
                    ret1.take().unwrap();
                let ret12 = ret12.take().unwrap();
                Poll::Ready((
                    ret1, ret2, ret3, ret4, ret5, ret6, ret7, ret8, ret9, ret10, ret11, ret12,
                ))
            }
            _ => Poll::Pending,
        }
    }
}
