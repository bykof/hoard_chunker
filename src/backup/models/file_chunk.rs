use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileChunk {
    pub hash: String,
    pub offset: u64,
    pub length: usize,
}
