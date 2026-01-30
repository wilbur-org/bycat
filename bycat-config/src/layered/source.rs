use std::collections::HashMap;

use bycat_error::Error;
use bycat_value::{Map, String};
use futures::future::BoxFuture;

pub trait ConfigSource {
    fn load_config<'a>(&'a self) -> BoxFuture<'a, Result<Map, Error>>;
    fn save_config<'a>(&'a self, config: &'a Map) -> BoxFuture<'a, Result<(), Error>>;
}
