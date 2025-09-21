use std::path::PathBuf;

pub struct CliRequest {
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

impl CliRequest {
    pub fn from_env() -> CliRequest {
        let cwd = std::env::current_dir().unwrap();
        let args = std::env::args().collect();
        CliRequest { args, cwd }
    }
}
