// mod command;
mod context;
mod error;
mod file;
mod geenie;
mod item;
// #[cfg(feature = "process")]
// mod process;
mod result;

pub mod questions {
    pub use spurgt::{
        Confirm, Input, MultiSelect, Password, Select, confirm, input, multi_select, password,
        select,
    };
}

pub mod ui {
    pub use spurgt::{ProgressBar, Spinner};
}

pub use self::{
    context::Context,
    error::GeenieError,
    file::FileList,
    geenie::Geenie,
    item::{Item, ItemExt, MountItem},
};

// #[cfg(feature = "process")]
// pub use self::process::*;

pub use relative_path;
