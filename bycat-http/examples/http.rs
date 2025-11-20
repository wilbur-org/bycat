use bycat::prelude::WorkExt;
use bycat_http::{WorkIntoResponseExt, extract::RequestBodyLimit, handler};

#[tokio::main(flavor = "current_thread")]
async fn main() -> bycat_error::Result<()> {
    bycat_http::serve(
        ("localhost", 3000),
        (),
        handler(|| async move { "Hello, World!" })
            .wrap(RequestBodyLimit(1024))
            .into_response(),
    )
    .await
    .unwrap();

    Ok(())
}
