use std::collections::HashMap;

pub struct Translation {
    messages: HashMap<String, String>,
}

impl Translation {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }

    pub fn from_hashmap(messages: HashMap<String, String>) -> Self {
        Self { messages }
    }

    fn get_first(&self) -> &str {
        self.messages.values().next().map(|s| s.as_str()).unwrap_or("")
    }

    pub fn get(&self, key: &str) -> &str {
        self.messages.get(key).map(|s| s.as_str()).unwrap_or(self.get_first())
    }
}

#[macro_export]
macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        core::convert::From::from([$(($k.to_string(), $v.to_string()),)*])
    }};
}

#[macro_export]
macro_rules! trad {
    ($($k:expr => $v:expr),* $(,)?) => {{
        Translation::from_hashmap(collection![$($k => $v),*])
    }};
}

pub use trad;
