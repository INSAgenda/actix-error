use std::{path::PathBuf, collections::HashMap};

pub fn get_po_error_messages(path: PathBuf) -> HashMap<String, HashMap<String, String>> {
    use poreader::{PoParser, Message}; 

    // Get list of .po files
    let mut po_files = Vec::new();
    for entry in std::fs::read_dir(path).expect("Path doesn't exist.") {
        let entry = entry.expect("Couldn't read entry.");
        let path = entry.path();
        if path.extension().expect("Couldn't get the file extension") == "po" {
            po_files.push(path);
        }
    }
    // Get the messages from the .po files
    let mut messages: HashMap<String, HashMap<String, String>> = HashMap::new();
    for po_file in po_files {
        let parser = PoParser::new();
        let file = std::fs::File::open(&po_file).expect("Couldn't open PO file.");
        let reader = parser.parse(file).expect("Couldn't parse PO file.");
        let key = &po_file.file_stem().expect("Couln't get filename.").to_str().expect("Couldn't convert filename to str").to_string();
        
        for unit in reader {
            let Ok(unit) = unit else {
                eprintln!("WARNING: Invalid unit in the {} catalog", &key);
                continue;
            };
            if let Message::Simple { id, text: Some(text) } = unit.message() {
                if let Some(msgs) = messages.get_mut(id) {
                    msgs.insert(key.to_owned(), text.to_owned());
                } else {
                    messages.insert(id.to_owned(), HashMap::from([(key.to_owned(), text.to_owned())]));
                }
            }
        }
    }
    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_get_po_error_messages() {
        let path = PathBuf::from("../locales");
        let messages = get_po_error_messages(path);
        assert_eq!(messages.len(), 4);
        assert_eq!(messages.get("named_error").unwrap().len(), 2);
        assert_eq!(messages.get("invalid_id").unwrap().len(), 2);
    }
}