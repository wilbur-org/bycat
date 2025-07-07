use crate::futures::*;
use core::future::Future;
use futures::executor::block_on;

struct TestFuture {
    value: i32,
}

impl Future for TestFuture {
    type Output = i32;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        core::task::Poll::Ready(self.value)
    }
}

#[test]
fn test_bycat_future() {
    let fut = TestFuture { value: 42 };
    let bycat_fut = BycatFuture::new(fut);

    let result = block_on(bycat_fut);
    assert_eq!(result, (42,));
}

#[test]
fn test_bycat_future2() {
    let fut1 = TestFuture { value: 42 };
    let fut2 = TestFuture { value: 24 };
    let bycat_fut = BycatFuture2::new(fut1, fut2);

    let result = block_on(bycat_fut);
    assert_eq!(result, (42, 24));
}

#[test]
fn test_bycat_future3() {
    let fut1 = TestFuture { value: 42 };
    let fut2 = TestFuture { value: 24 };
    let fut3 = TestFuture { value: 84 };
    let bycat_fut = BycatFuture3::new(fut1, fut2, fut3);

    let result = block_on(bycat_fut);
    assert_eq!(result, (42, 24, 84));
}

// Add similar tests for BycatFuture4, BycatFuture5, etc., as needed.
