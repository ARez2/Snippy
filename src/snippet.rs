use serde_derive::{Serialize, Deserialize};


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeSnippet {
    pub tags: Vec<String>,
    pub name: String,
    pub code: String,
    pub idx: usize,
}
impl CodeSnippet {
    pub fn new(new_idx: usize) -> CodeSnippet {
        CodeSnippet { tags: vec![],
            name: "Unnamed Code Snippet".to_string(),
            code: "".to_string(),
            idx: new_idx,
        }
    }
}