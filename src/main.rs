pub mod backup;

use core::str;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};

use hoard_chunker::DEFAULT_AVERAGE_SIZE;
use log::{info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};

use crate::backup::{
    backup_config::BackupConfig, backup_metadata::BackupMetadata, backup_service::BackupService,
    chunk_writer::ChunkWriter,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    average_size: Option<u32>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Backup {
        #[arg(short, long)]
        input_path: PathBuf,

        #[arg(short, long)]
        output_path: PathBuf,
    },
    Restore {
        #[arg(short, long)]
        input_path: PathBuf,

        #[arg(short, long)]
        output_path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // 16KB
    let average_size = cli.average_size.unwrap_or(DEFAULT_AVERAGE_SIZE);
    info!("Average size is {}", average_size);

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    log::set_max_level(LevelFilter::Debug);

    match &cli.command {
        Some(Commands::Backup {
            input_path,
            output_path,
        }) => {
            let backup_config = &BackupConfig::new(DEFAULT_AVERAGE_SIZE, input_path, output_path);
            let chunk_writer = ChunkWriter::new(backup_config);
            let mut backup_metadata = BackupMetadata::new();
            let mut backup = BackupService::new(backup_config, &chunk_writer, &mut backup_metadata);
            backup.backup()?;
        }
        Some(Commands::Restore {
            input_path,
            output_path,
        }) => {
            let backup_config = BackupConfig::new(DEFAULT_AVERAGE_SIZE, input_path, output_path);
            let chunk_writer = ChunkWriter::new(&backup_config);
            let mut backup_metadata =
                BackupMetadata::deserialize(Path::new(&backup_config.input_path.clone()))?;
            let backup = BackupService::new(&backup_config, &chunk_writer, &mut backup_metadata);
            backup.restore()?;
        }
        None => {}
    }

    Ok(())
}
