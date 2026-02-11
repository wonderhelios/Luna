use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CodeChunk {
    pub path: String,
    #[serde(rename = "alias")]
    pub alias: usize,
    #[serde(rename = "snippet")]
    pub snippet: String,
    #[serde(rename = "start")]
    pub start_line: usize,
    #[serde(rename = "end")]
    pub end_line: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct CodeChunkMiddle<'a> {
    pub path: &'a str,
    pub text: &'a str,
    pub alias: usize,
    pub start_line: usize,
    pub end_line: usize,
}

impl CodeChunkMiddle<'_> {
    /// Returns true if a code-chunk contains an empty snippet or a snippet with only whitespace
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

impl fmt::Display for CodeChunkMiddle<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{}", self.path, self.text)
    }
}

impl CodeChunk {
    /// Returns true if a code-chunk contains an empty snippet or a snippet with only whitespace
    pub fn is_empty(&self) -> bool {
        self.snippet.trim().is_empty()
    }
    pub fn to_middle(&self) -> CodeChunkMiddle<'_> {
        CodeChunkMiddle {
            path: &self.path,
            text: &self.snippet,
            alias: self.alias,
            start_line: self.start_line,
            end_line: self.end_line,
        }
    }
}

impl fmt::Display for CodeChunk {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}\n{}", self.alias, self.path, self.snippet)
    }
}
