use core::str;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use fastcdc::v2020::ChunkData;

pub fn split_hash(hash: u64) -> Vec<String> {
    return format!("{:X}", hash)
        .as_bytes()
        .chunks(2)
        .map(str::from_utf8)
        .map(Result::unwrap)
        .map(String::from)
        .collect();
}

pub fn store_chunk(chunk: &ChunkData, output_path: &Path) {
    let mut source_path = PathBuf::from(output_path);
    for item in split_hash(chunk.hash) {
        source_path.push(item);
    }

    if let Some(parent) = source_path.parent() {
        // Create the directories "as/df/gh"
        fs::create_dir_all(parent).expect("cannot create directories");

        let mut file = fs::File::create(source_path).expect("cannot write the file");
        file.write_all(chunk.data.as_slice())
            .expect("cannot write file");
    }
}
