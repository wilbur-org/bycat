mod app;
mod builder;
mod cli;
mod logging;
pub mod paths;
mod req;

pub use self::{app::*, builder::*, paths::Paths, req::CliRequest};

pub mod config {
    pub use bycat_config::{Config, Mode};
}

pub use bycat_error::{Error, Result};

pub mod prelude {
    pub use bycat_fs::VirtualFS;
    pub use bycat_source::{Source, SourceExt};
}
