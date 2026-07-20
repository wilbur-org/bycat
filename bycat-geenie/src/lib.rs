mod command;
mod context;
mod error;
mod file;
mod geenie;
mod item;
#[cfg(feature = "process")]
mod process;
mod result;

pub mod questions {
    pub use spurgt::{
        confirm, input, multi_select, password, select, Confirm, Input, MultiSelect, Password,
        Select,
    };
}

pub mod ui {
    pub use spurgt::{ProgressBar, Spinner};
}

#[cfg(feature = "cli")]
pub use spurgt_cliclack::Cliclack as Cli;

pub use self::{
    command::{Command, DynamicCommand},
    context::Context,
    error::GeenieError,
    file::{File, FileList},
    geenie::Geenie,
    item::{Item, ItemExt, MountItem},
};

#[cfg(feature = "process")]
pub use self::process::*;

pub use relative_path;
