use std::{collections::HashMap, path::PathBuf};
use serde_json;


pub fn get_json_error_messages(path: PathBuf) -> HashMap<String, HashMap<String, String>> {
    // Get the json file   
    let file = std::fs::File::open(&path).expect("Couldn't open JSON file.");
    
    // Get the messages from the json file
    let messages: HashMap<String, HashMap<String, String>> = serde_json::from_reader(file).expect("Couldn't parse JSON file.");

    messages
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_json_error_messages() {
        let path = PathBuf::from("../locales/messages.json");
        let messages = get_json_error_messages(path);
        assert_eq!(messages.len(), 4);
        assert_eq!(messages.get("named_error").unwrap().len(), 2);
        assert_eq!(messages.get("invalid_id").unwrap().len(), 2);
    }
}