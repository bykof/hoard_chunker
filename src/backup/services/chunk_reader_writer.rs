use crate::backup::models::lib::split_hash_as_path;
use anyhow::Result;
use opendal::layers::{LoggingLayer, RetryLayer};
use opendal::services::Fs;
use opendal::{BlockingOperator, Operator};
use std::path::Path;

pub struct ChunkReaderWriter {}

impl ChunkReaderWriter {
    pub fn new() -> ChunkReaderWriter {
        ChunkReaderWriter {}
    }

    pub fn build_operator(&self) -> Result<BlockingOperator> {
        let builder = Fs::default().root("./");

        Ok(Operator::new(builder)?
            .layer(LoggingLayer::default())
            .layer(RetryLayer::new())
            .finish()
            .blocking())
    }

    pub fn write_chunk(&self, hash: &str, data: &Vec<u8>, output_dir: &Path) -> Result<()> {
        let operator = self.build_operator()?;
        let file_path = split_hash_as_path(output_dir, hash.to_string());
        let compressed_data = zstd::encode_all(data.as_slice(), 1)?;
        Ok(operator.write(file_path.to_str().unwrap(), compressed_data)?)
    }

    pub fn read_chunk(&self, hash: &str, input_dir: &Path) -> Result<Vec<u8>> {
        let operator = self.build_operator()?;
        let file_path = split_hash_as_path(input_dir, hash.to_string());
        let compressed_data = operator.read(file_path.to_str().unwrap())?.to_vec();
        Ok(zstd::decode_all(compressed_data.as_slice())?.to_vec())
    }
}
