use crate::env::Environ;
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Request {
    args: Vec<String>,
    env: Environ,
    #[cfg(feature = "std")]
    cwd: PathBuf,
}

impl Request {
    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub fn env(&self) -> &Environ {
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut Environ {
        &mut self.env
    }
}

#[cfg(feature = "std")]
impl Request {
    pub fn from_env() -> std::io::Result<Request> {
        let cwd = std::env::current_dir()?;
        Ok(Request {
            args: std::env::args().collect(),
            env: Environ::from_env(),
            cwd,
        })
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }
}
