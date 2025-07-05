use crate::futures::*;
use core::future::Future;

impl<T1> From<(T1,)> for BycatFuture<T1>
where
    T1: Future,
{
    fn from(fut1: (T1,)) -> Self {
        BycatFuture::new(fut1.0)
    }
}

impl<T1, T2> From<(T1, T2)> for BycatFuture2<T1, T2>
where
    T1: Future,
    T2: Future,
{
    fn from((fut1, fut2): (T1, T2)) -> Self {
        BycatFuture2::new(fut1, fut2)
    }
}

impl<T1, T2, T3> From<(T1, T2, T3)> for BycatFuture3<T1, T2, T3>
where
    T1: Future,
    T2: Future,
    T3: Future,
{
    fn from((fut1, fut2, fut3): (T1, T2, T3)) -> Self {
        BycatFuture3::new(fut1, fut2, fut3)
    }
}

impl<T1, T2, T3, T4> From<(T1, T2, T3, T4)> for BycatFuture4<T1, T2, T3, T4>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
{
    fn from((fut1, fut2, fut3, fut4): (T1, T2, T3, T4)) -> Self {
        BycatFuture4::new(fut1, fut2, fut3, fut4)
    }
}

impl<T1, T2, T3, T4, T5> From<(T1, T2, T3, T4, T5)> for BycatFuture5<T1, T2, T3, T4, T5>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
{
    fn from((fut1, fut2, fut3, fut4, fut5): (T1, T2, T3, T4, T5)) -> Self {
        BycatFuture5::new(fut1, fut2, fut3, fut4, fut5)
    }
}

impl<T1, T2, T3, T4, T5, T6> From<(T1, T2, T3, T4, T5, T6)> for BycatFuture6<T1, T2, T3, T4, T5, T6>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
{
    fn from((fut1, fut2, fut3, fut4, fut5, fut6): (T1, T2, T3, T4, T5, T6)) -> Self {
        BycatFuture6::new(fut1, fut2, fut3, fut4, fut5, fut6)
    }
}

impl<T1, T2, T3, T4, T5, T6, T7> From<(T1, T2, T3, T4, T5, T6, T7)>
    for BycatFuture7<T1, T2, T3, T4, T5, T6, T7>
where
    T1: Future,
    T2: Future,
    T3: Future,
    T4: Future,
    T5: Future,
    T6: Future,
    T7: Future,
{
    fn from((fut1, fut2, fut3, fut4, fut5, fut6, fut7): (T1, T2, T3, T4, T5, T6, T7)) -> Self {
        BycatFuture7::new(fut1, fut2, fut3, fut4, fut5, fut6, fut7)
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8> From<(T1, T2, T3, T4, T5, T6, T7, T8)>
    for BycatFuture8<T1, T2, T3, T4, T5, T6, T7, T8>
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
    fn from(
        (fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8): (T1, T2, T3, T4, T5, T6, T7, T8),
    ) -> Self {
        BycatFuture8::new(fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8)
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9> From<(T1, T2, T3, T4, T5, T6, T7, T8, T9)>
    for BycatFuture9<T1, T2, T3, T4, T5, T6, T7, T8, T9>
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
    fn from(
        (fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9): (
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
        ),
    ) -> Self {
        BycatFuture9::new(fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9)
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10> From<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)>
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
    fn from(
        (fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9, fut10): (
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
            T10,
        ),
    ) -> Self {
        BycatFuture10::new(fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9, fut10)
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>
    From<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)>
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
    fn from(
        (fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9, fut10, fut11): (
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
            T10,
            T11,
        ),
    ) -> Self {
        BycatFuture11::new(
            fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9, fut10, fut11,
        )
    }
}

impl<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>
    From<(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12)>
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
    fn from(
        (fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9, fut10, fut11, fut12): (
            T1,
            T2,
            T3,
            T4,
            T5,
            T6,
            T7,
            T8,
            T9,
            T10,
            T11,
            T12,
        ),
    ) -> Self {
        BycatFuture12::new(
            fut1, fut2, fut3, fut4, fut5, fut6, fut7, fut8, fut9, fut10, fut11, fut12,
        )
    }
}
