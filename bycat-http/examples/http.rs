use bycat::prelude::WorkExt;
use bycat_http::{
    WorkIntoResponseExt,
    cookies::Cookies,
    extract::RequestBodyLimit,
    handler,
    session::{MemoryStore, Session, Sessions},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> bycat_error::Result<()> {
    bycat_http::serve(
        ("localhost", 3000),
        (),
        handler(|mut session: Session| async move {
            let value: u64 = session.get("counter").map(|m| m.unwrap_or_default())?;
            session.set("counter", value + 1);

            session.regenerate_id().await?;

            bycat_error::Result::Ok(format!("Count: {}", value))
        })
        .wrap(RequestBodyLimit(1024))
        .wrap(Sessions::new(MemoryStore::default()))
        .wrap(Cookies)
        .into_response(),
    )
    .await
    .unwrap();

    Ok(())
}
