use std::collections::HashMap;
use tower_lsp::lsp_types::Url;

/// In-memory store for files currently opened in the IDE.
pub struct DocumentStore {
    documents: HashMap<Url, String>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn update_document(&mut self, uri: Url, text: String) {
        self.documents.insert(uri, text);
    }

    pub fn get_document(&self, uri: &Url) -> Option<&String> {
        self.documents.get(uri)
    }
}