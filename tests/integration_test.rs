use anyhow::Result;
use hoard_chunker::backup::models::backup_config::BackupConfig;
use hoard_chunker::backup::services::backup_service::BackupService;
use hoard_chunker::backup::services::chunk_reader_writer::ChunkReaderWriter;
use hoard_chunker::backup::services::chunk_storage::{ChunkStorage, LocalChunkStorage};
use hoard_chunker::backup::services::file_chunker::FileChunker;
use hoard_chunker::DEFAULT_AVERAGE_SIZE;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::sync::Arc;
use std::{fs, path::Path};
use walkdir::WalkDir;

#[test]
fn test_backup_and_restore() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;
    let threads = 4;
    let chunk_reader_writer = Arc::new(ChunkReaderWriter::new());
    let chunk_storage: Arc<Box<dyn ChunkStorage + Send + Sync + 'static>> =
        Arc::new(Box::new(LocalChunkStorage::new()));

    let backup_input_path = "./tests/assets";
    let backup_output_path = "./target/output";
    let backup_config = Arc::new(BackupConfig::new(
        DEFAULT_AVERAGE_SIZE,
        backup_input_path.as_ref(),
        backup_output_path.as_ref(),
        threads,
    ));
    let backup_file_chunker = Arc::new(FileChunker::new(
        backup_config.clone(),
        chunk_reader_writer.clone(),
        chunk_storage.clone(),
    ));
    let mut backup_service = BackupService::new(
        backup_config.clone(),
        backup_file_chunker.clone(),
        chunk_reader_writer.clone(),
        chunk_storage.clone(),
    );
    backup_service.backup()?;

    let restore_input_path = Path::new(backup_output_path);
    let restore_output_path = Path::new("./target/restored");
    let restore_config = Arc::new(BackupConfig::new(
        DEFAULT_AVERAGE_SIZE,
        &restore_input_path,
        &restore_output_path,
        threads,
    ));
    let restore_file_chunker = Arc::new(FileChunker::new(
        restore_config.clone(),
        chunk_reader_writer.clone(),
        chunk_storage.clone(),
    ));
    let mut backup_service = BackupService::new(
        restore_config.clone(),
        restore_file_chunker.clone(),
        chunk_reader_writer.clone(),
        chunk_storage.clone(),
    );
    backup_service.restore()?;

    for entry in WalkDir::new(restore_output_path) {
        let dir_entry = entry?;
        if dir_entry.path().is_file() {
            let compare_to_filepath = dir_entry.path().strip_prefix(restore_output_path)?;
            info!(
                "Comparing {} with {}",
                dir_entry.path().display(),
                compare_to_filepath.display()
            );
            assert_eq!(fs::read(compare_to_filepath)?, fs::read(dir_entry.path())?)
        }
    }
    Ok(())
}
