use bycat_http::{error::Result, handler};
use tokio::task::LocalSet;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let local_set = LocalSet::new();

    local_set
        .run_until(async move {
            bycat_http::serve::Tokio::new(handler(|| async move { "Hello, World!" }))
                .serve((), ("localhost", 3000))
                .await
        })
        .await?;

    Ok(())
}
