use std::path::{Path, PathBuf};

pub fn split_hash(hash: &str) -> &str {
    &hash[0..2]
}
pub fn split_hash_as_path(prefix_path: &Path, hash: String) -> PathBuf {
    PathBuf::from(prefix_path)
        .join(split_hash(&hash))
        .join(hash)
}
