use crate::backup::models::lib::split_hash_as_path;
use anyhow::Result;
use opendal::layers::{LoggingLayer, RetryLayer};
use opendal::services::Fs;
use opendal::{BlockingOperator, Buffer, Operator};
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

    pub fn write_chunk(
        &self,
        hash: &str,
        data: impl Into<Buffer>,
        output_dir: &Path,
    ) -> Result<()> {
        let operator = self.build_operator()?;
        let file_path = split_hash_as_path(output_dir, hash.to_string());
        Ok(operator.write(file_path.to_str().unwrap(), data)?)
    }

    pub fn read_chunk(&self, hash: &str, input_dir: &Path) -> Result<Vec<u8>> {
        let operator = self.build_operator()?;
        let file_path = split_hash_as_path(input_dir, hash.to_string());
        Ok(operator.read(file_path.to_str().unwrap())?.to_vec())
    }
}
