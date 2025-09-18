use std::convert::Infallible;

use bycat::work_fn;
use bycat_source::{iter, pipe, prelude::*, Source};
use futures::TryStreamExt;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let pipe = pipe(iter(vec![Result::<_, Infallible>::Ok("Hello")]))
        .pipe(work_fn(|ctx, pkg| async move {
            println!("Work {pkg}");
            Result::<_, Infallible>::Ok("Other")
        }))
        .pipe(work_fn(|ctx, pkg| async move {
            println!("Work 2: {pkg}");
            Result::<_, Infallible>::Ok("next other")
        }))
        .cloned(
            work_fn(|ctx, req| async { Result::<_, Infallible>::Ok("Cloned 1") }),
            work_fn(|ctx, req| async { Result::<_, Infallible>::Ok("Cloned 2") }),
        );

    pipe.create_stream(&())
        .try_for_each(|rx| async move {
            //
            println!("Output {}", rx);
            Ok(())
        })
        .await
        .unwrap();

    // pipe.run(()).await;
}
