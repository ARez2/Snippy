use std::collections::HashMap;

use serde_derive::{Serialize, Deserialize};
use tui::widgets::ListState;
pub mod snippet;
pub mod app;

#[derive(Serialize, Deserialize)]
pub struct SnippyConfig {
    pub keys: HashMap<String, char>,
}

impl Default for SnippyConfig {
    fn default() -> Self {
        let mut keys = HashMap::new();
        keys.insert("KEY_NEW".to_string(), 'n');
        keys.insert("KEY_FIND".to_string(), 'f');
        keys.insert("KEY_SAVESNIPPET".to_string(), 's');
        keys.insert("KEY_COPY".to_string(), 'c');
        keys.insert("KEY_DELETE".to_string(), 'x');
        SnippyConfig {
            keys,
        }
    }
}









#[derive(Clone)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {return};
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {return};

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        StatefulList {
            state: ListState::default(),
            items: Vec::<T>::new(),
        }
    }
}