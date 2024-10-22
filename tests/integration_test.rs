use std::{fs, path::Path};

use anyhow::Result;
use hoard_chunker::{
    backup::{
        backup_config::BackupConfig, backup_metadata::BackupMetadata,
        backup_service::BackupService, chunk_writer::ChunkWriter,
    },
    DEFAULT_AVERAGE_SIZE,
};
use log::{info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use walkdir::WalkDir;

#[test]
fn test_backup_and_restore() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();
    let backup_input_path = "./tests/assets";
    let backup_target_path = "./target/output";
    let backup_config = &BackupConfig::new(
        DEFAULT_AVERAGE_SIZE,
        Path::new(backup_input_path),
        Path::new(backup_target_path),
    );
    let chunk_writer = ChunkWriter::new(backup_config);
    let mut backup_metadata = BackupMetadata::new();
    let mut backup_service = BackupService::new(backup_config, &chunk_writer, &mut backup_metadata);
    backup_service.backup()?;

    let restore_input_path = Path::new(backup_target_path);
    let restore_output_path = Path::new("./target/restored");
    let restore_config = &BackupConfig::new(
        DEFAULT_AVERAGE_SIZE,
        &restore_input_path,
        &restore_output_path,
    );
    let restore_chunk_writer = ChunkWriter::new(restore_config);
    let mut restore_metadata = BackupMetadata::deserialize(&restore_input_path)?;
    let restore_backup_service =
        BackupService::new(restore_config, &restore_chunk_writer, &mut restore_metadata);
    restore_backup_service.restore()?;

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
