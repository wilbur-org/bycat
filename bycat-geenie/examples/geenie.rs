use std::{future, str::FromStr};

use bycat_geenie::{Context, GeenieError};
use bycat_package::{Mime, Package};

fn main() -> Result<(), GeenieError> {
    futures::executor::block_on(async {
        let mut geenie = bycat_geenie::Geenie::<(), String>::new();

        geenie.push(async move |mut ctx: Context<'_, (), String>| {
            // ctx.info("Hello, world!").await?;
            ctx.package(Package::new(
                "main.rs",
                Mime::from_str("text/rust")?,
                "fn main() {}".into(),
            ))?;

            Ok(())
        });

        let result = geenie.run(&mut ()).await?;

        for file in result.files {
            println!("File: {}", file.path());
            println!("Content: {}", &file.content());
        }

        Ok(())
    })
}
