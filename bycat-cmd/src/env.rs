use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Environ {
    entries: HashMap<String, String>,
}

impl Environ {
    pub fn from_env() -> Environ {
        Environ {
            entries: HashMap::from_iter(std::env::vars()),
        }
    }
}
