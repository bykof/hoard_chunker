use std::path::PathBuf;

pub fn split_hash(hash: String) -> Vec<String> {
    return hash
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .map(|chunk| chunk.iter().collect())
        .collect();
}

pub fn split_hash_as_path(prefix_path: String, hash: String) -> PathBuf {
    let mut path = PathBuf::from(prefix_path);
    split_hash(hash)
        .iter()
        .for_each(|splitted_hash| path.push(splitted_hash));
    return path;
}
