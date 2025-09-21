use bycat::work_fn;
use bycat_cli::App;
use bycat_config::Mode;
use bycat_error::Error;
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
            println!("App: {:?}", app.paths().config());
            Result::<_, Error>::Ok(())
        }))?
        .run()
        .await?;

    Ok(())
}
