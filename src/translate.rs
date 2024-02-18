use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Translation {
    messages: HashMap<String, String>,
}

impl Translation {
    pub fn from_hashmap(messages: HashMap<String, String>) -> Self {
        Self { messages }
    }

    fn get_first(&self) -> &str {
        self.messages
            .values()
            .next()
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn get(&self, key: &str) -> &str {
        self.messages
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(self.get_first())
    }
}


#[macro_export]
macro_rules! tr {
    ($($k:expr => $v:expr),* $(,)?) => {{
        Translation::from_hashmap(core::convert::From::from([$(($k.to_string(), $v.to_string()),)*]))
    }};
}
pub use tr;
