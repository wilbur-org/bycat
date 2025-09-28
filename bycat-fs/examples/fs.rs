use std::path::PathBuf;

use bycat::work_fn;
use bycat_error::Error;
use bycat_fs::WalkDir;
use bycat_package::{Decode, Package, match_glob};
use bycat_source::{Unit, pipe, prelude::*};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Test {
    rustc_fingerprint: u64,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let fs = pipe(WalkDir::new(PathBuf::from(".")).pattern(match_glob("**/*.json")))
        .pipe(Decode::new())
        .pipe(work_fn(|_, pkg: Package<Test>| async move {
            //
            println!("{}", pkg.name());
            Result::<_, Error>::Ok(())
        }))
        .then(work_fn(|ctx, ret: Result<(), Error>| async move {
            if let Err(err) = ret.as_ref() {
                // println!("{:?}", err);
            }
            ret
        }))
        .unit()
        .run(&())
        .await;
}
