use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
};

#[derive(Debug, Default, Clone)]
pub struct Environ {
    entries: BTreeMap<String, String>,
}

impl Environ {
    #[cfg(feature = "std")]
    pub fn from_env() -> Environ {
        Environ {
            entries: BTreeMap::from_iter(std::env::vars()),
        }
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.entries.get(name).map(|m| m.as_str())
    }

    pub fn set(&mut self, name: impl ToString, value: impl ToString) -> Option<String> {
        self.entries.insert(name.to_string(), value.to_string())
    }
}
