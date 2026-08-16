use crate::{
    Middleware, Service, and::And, map::Map, map_err::MapErr, split::Split, then::Then,
    util::IntoEither,
};

pub trait ServiceExt<C, I>: Service<C, I> {
    fn pipe<T>(self, next: T) -> And<Self, T>
    where
        Self: Sized,
        T: Service<C, Self::Output>,
    {
        And::new(self, next)
    }

    fn then<T>(self, next: T) -> Then<Self, T>
    where
        Self: Sized,
        T: Service<C, Result<Self::Output, Self::Error>>,
    {
        Then::new(self, next)
    }

    fn split<L, R>(self, left: L, right: R) -> Split<Self, L, R>
    where
        Self: Sized,
        Self::Output: IntoEither,
        L: Service<C, <Self::Output as IntoEither>::Left, Error = Self::Error> + Clone,
        R: Service<C, <Self::Output as IntoEither>::Right, Output = L::Output, Error = Self::Error>
            + Clone,
        C: Clone,
    {
        Split::new(self, left, right)
    }

    fn map_err<T, E>(self, map: T) -> MapErr<Self, T, E>
    where
        Self: Sized,
        T: Fn(Self::Error) -> E,
    {
        MapErr::new(self, map)
    }

    fn map<T, O>(self, map: T) -> Map<Self, T, O>
    where
        Self: Sized,
        T: Fn(Self::Output) -> O,
    {
        Map::new(self, map)
    }

    fn wrap<M>(self, middleware: M) -> M::Work
    where
        Self: Sized,
        M: Middleware<C, I, Self>,
    {
        middleware.wrap(self)
    }
}

impl<C, I, T> ServiceExt<C, I> for T where T: Service<C, I> {}
