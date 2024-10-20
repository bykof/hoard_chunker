use std::{collections::HashMap, hash::Hash};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct FileChunk {
    pub hash: String,
    pub offset: u64,
    pub length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub root_path: String,
    // hash -> FileChunk
    pub chunks: HashMap<String, FileChunk>,
}

impl FileMetadata {
    pub fn fingerprint(&self) -> String {
        let mut hasher = blake3::Hasher::new();

        self.chunks
            .iter()
            .sorted_by(|(_, file_chunk_a), (_, file_chunk_b)| {
                Ord::cmp(&file_chunk_a.offset, &file_chunk_b.offset)
            })
            .for_each(|(hash, _)| {
                hasher.update(hash.to_string().as_bytes());
            });

        hasher.finalize().to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_metadata_fingerprint_equal() {
        let hashes = Vec::from([123, 456, 789]);
        let mut chunks: HashMap<u64, FileChunk> = HashMap::new();
        let mut second_chunks: HashMap<u64, FileChunk> = HashMap::new();

        for (index, hash) in hashes.iter().enumerate() {
            chunks.insert(
                hash.clone(),
                FileChunk {
                    hash: hash.clone(),
                    offset: index.clone() as u64,
                    length: 8,
                },
            );
        }

        for (index, hash) in hashes.iter().enumerate().rev() {
            second_chunks.insert(
                hash.clone(),
                FileChunk {
                    hash: hash.clone(),
                    offset: index.clone() as u64,
                    length: 8,
                },
            );
        }

        let file_metadata = FileMetadata {
            root_path: "".to_string(),
            chunks: chunks,
        };

        let second_file_metadata = FileMetadata {
            root_path: "".to_string(),
            chunks: second_chunks,
        };
        assert_eq!(
            file_metadata.fingerprint(),
            second_file_metadata.fingerprint()
        )
    }

    #[test]
    fn file_metadata_fingerprint_not_equal() {
        let hashes = Vec::from([123, 456, 789]);
        let other_hashes = Vec::from([234, 567, 890]);
        let mut chunks: HashMap<u64, FileChunk> = HashMap::new();
        let mut second_chunks: HashMap<u64, FileChunk> = HashMap::new();

        for (index, hash) in hashes.iter().enumerate() {
            chunks.insert(
                hash.clone(),
                FileChunk {
                    hash: hash.clone(),
                    offset: index.clone() as u64,
                    length: 8,
                },
            );
        }

        for (index, hash) in other_hashes.iter().enumerate() {
            second_chunks.insert(
                hash.clone(),
                FileChunk {
                    hash: hash.clone(),
                    offset: index.clone() as u64,
                    length: 8,
                },
            );
        }

        let file_metadata = FileMetadata {
            root_path: "".to_string(),
            chunks: chunks,
        };

        let second_file_metadata = FileMetadata {
            root_path: "".to_string(),
            chunks: second_chunks,
        };
        assert_ne!(
            file_metadata.fingerprint(),
            second_file_metadata.fingerprint()
        )
    }
}
