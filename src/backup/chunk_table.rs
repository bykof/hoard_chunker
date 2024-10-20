use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub hash: u64,
    pub length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkTable {
    // chunk hash -> Chunk
    pub chunk_map: HashMap<u64, Chunk>,
}
