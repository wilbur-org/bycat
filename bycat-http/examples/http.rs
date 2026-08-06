use bycat::prelude::WorkExt;
use bycat_http::{
    cookies::Cookies,
    cors::Cors,
    error::Result,
    extract::RequestBodyLimit,
    handler,
    prelude::HttpWorkExt,
    session::{MemoryStore, Session, Sessions},
};
use http::Request;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    bycat_http::serve(
        ("localhost", 3000),
        (),
        handler(|mut session: Session| async move {
            let value: u64 = session.get("counter").map(|m| m.unwrap_or_default())?;
            session.set("counter", value + 1);

            session.regenerate_id().await?;

            Result::Ok(format!("Count: {}", value))
        })
        .wrap(RequestBodyLimit(1024))
        .wrap(Sessions::new(MemoryStore::default()))
        .wrap(Cookies)
        .wrap(Cors::default())
        .with_filter(|req: &Request<_>| req.uri().path() == "/")
        .or(handler(async || "Other!!")
            .with_filter(|req: &Request<_>| req.uri().path() == "/other")),
    )
    .await
    .unwrap();

    Ok(())
}
