use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case", tag = "type", content = "content")]
pub enum Definition {
    FunctionDef {
        file_path: String,
        line_number: usize,
        name: String,
    },
    ClassDef {
        file_path: String,
        line_number: usize,
        name: String,
    },
    //TODO add enum
}
