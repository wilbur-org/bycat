use bycat_http::{error::Result, handler};

#[compio::main]
async fn main() -> Result<()> {
    bycat_http::serve::Compio::new(handler(|| async move { "Hello, World!" }))
        .serve((), ("localhost", 3000))
        .await
        .unwrap();

    Ok(())
}
