use bycat::work_fn;
use bycat_cli::{App, prelude::*};
use bycat_config::Mode;
use bycat_error::Error;
use bycat_package::match_glob;
use futures::{StreamExt, TryStreamExt};
use tracing::Level;

#[tokio::main(flavor = "current_thread")]
async fn main() -> bycat_error::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    App::new("cli")
        .config(|cfg| {
            cfg.set_local(Mode::Single, "test.config.{ext}".to_string());
        })
        .build(work_fn(|ctx: (), app: App| async move {
            println!(
                "App: {:?}",
                app.paths()
                    .config()
                    .list(".")
                    .create_stream(&())
                    .map_ok(|m| m.path().to_relative_path_buf())
                    .try_collect::<Vec<_>>()
                    .await
            );
            Result::<_, Error>::Ok(())
        }))?
        .run()
        .await?;

    Ok(())
}
