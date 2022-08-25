use std::collections::VecDeque;
use serde_derive::{Serialize, Deserialize};
use crate::{snippet::CodeSnippet, StatefulList};


#[derive(Clone, Copy, PartialEq)]
pub enum NewSnippetMode {
    TypeName,
    TypeTags,
    TypeCode,
}


#[derive(Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    NewSnippet(NewSnippetMode),
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Search
    }
}


/// App holds the state of the application
#[derive(Clone, Serialize, Deserialize)]
pub struct App {
    #[serde(skip_serializing, skip_deserializing)]
    pub input: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub input_mode: InputMode,
    
    pub snippets: Vec<CodeSnippet>,
    
    // When deleting an element from the middle of the snippet list
    // this idx becomes open and the next new snippet will get that idx
    pub open_idxs: VecDeque<usize>,
    
    /// Found snippets displayed when searching
    #[serde(skip_serializing, skip_deserializing)]
    pub found_snippets: StatefulList<CodeSnippet>,
    
    // Currently edited snippet
    #[serde(skip_serializing, skip_deserializing)]
    pub current_snippet: Option<CodeSnippet>,
}

impl Default for App {
    fn default() -> App {
        let mut app = App {
            input: String::new(),
            input_mode: InputMode::Search,
            snippets: vec![],
            open_idxs: VecDeque::new(),
            found_snippets: StatefulList::with_items(vec![]),
            current_snippet: None,
        };

        let mut example_snippet = CodeSnippet::new(app.return_next_idx());
        example_snippet.name = "Example Snippet #1".to_string();
        example_snippet.code = "enum InputMode {
            Normal,
            Search,
            NewSnippet,
        }".to_string();
        example_snippet.tags = vec!["example".to_string()];
        app.snippets.push(example_snippet);
        let mut example_snippet2 = CodeSnippet::new(app.return_next_idx());
        example_snippet2.name = "Example Snippet #2".to_string();
        example_snippet2.code = "func hello():
            print(hey bro)".to_string();
        example_snippet2.tags = vec!["example".to_string(), "bro".to_string()];
        app.snippets.push(example_snippet2);
        let mut example_snippet3 = CodeSnippet::new(app.return_next_idx());
        example_snippet3.name = "Example Snippet #3".to_string();
        example_snippet3.code = "func hello():
            print(hey bro)".to_string();
            example_snippet3.tags = vec!["example".to_string(), "bro".to_string()];
        app.snippets.push(example_snippet3);
        
        app
    }
}

impl App {
    /// Immutable version of return_next_index
    pub fn get_next_idx(&self) -> usize {
        if !self.snippets.is_empty() && !self.open_idxs.is_empty() {
            *self.open_idxs.front().unwrap()
        } else {
            self.snippets.len()
        }
    }
    /// Mutable version of get_next_idx
    pub fn return_next_idx(&mut self) -> usize {
        if !self.snippets.is_empty() && !self.open_idxs.is_empty() {
            self.open_idxs.pop_front().unwrap()
        } else {
            self.snippets.len()
        }
    }

    pub fn remove_snippet(&mut self, snippet_idx: usize) {
        let index = self.snippets.iter().position(|r| r.idx == snippet_idx).unwrap();
        self.snippets.remove(index);
        self.open_idxs.push_back(snippet_idx);
    }

    pub fn has_snippet_with_idx(&self, snippet_idx: usize) -> bool {
        for snip in self.snippets.iter() {
            if snip.idx == snippet_idx {
                return true;
            }
        }
        false
    }
}