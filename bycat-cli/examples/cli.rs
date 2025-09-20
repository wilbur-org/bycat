use bycat::work_fn;
use bycat_cli::{App, Builder};
use bycat_error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Builder::new("cli").build(work_fn(|ctx: (), app: App| async move {
        //
        println!("App: {:?}", app.paths().config());
        Result::<_, Error>::Ok(())
    }));

    app.run().await.unwrap();
}
