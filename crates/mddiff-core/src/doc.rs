use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocState {
    pub path: Option<PathBuf>,
    pub text: String,
    pub dirty: bool,
}

impl DocState {
    pub fn empty() -> Self {
        Self {
            path: None,
            text: String::new(),
            dirty: false,
        }
    }

    pub fn from_text(text: String, path: Option<PathBuf>) -> Self {
        Self {
            path,
            text,
            dirty: false,
        }
    }
}
