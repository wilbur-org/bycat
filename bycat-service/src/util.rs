use either::Either;

pub trait IntoEither {
    type Left;
    type Right;

    fn into_either(self) -> Either<Self::Left, Self::Right>;
}

impl<L, R> IntoEither for Either<L, R> {
    type Left = L;
    type Right = R;
    fn into_either(self) -> Either<L, R> {
        self
    }
}

impl<L, T> IntoEither for Result<L, T> {
    type Left = L;
    type Right = T;
    fn into_either(self) -> Either<L, T> {
        match self {
            Ok(l) => Either::Left(l),
            Err(r) => Either::Right(r),
        }
    }
}
