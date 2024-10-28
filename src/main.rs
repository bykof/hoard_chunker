extern crate num_cpus;
pub mod backup;

use anyhow::Result;
use clap::{Parser, Subcommand};
use core::str;
use hoard_chunker::backup::models::backup_config::BackupConfig;
use hoard_chunker::backup::services::backup_service::BackupService;
use hoard_chunker::backup::services::chunk_reader_writer::ChunkReaderWriter;
use hoard_chunker::backup::services::chunk_storage::{ChunkStorage, LocalChunkStorage};
use hoard_chunker::backup::services::file_chunker::FileChunker;
use hoard_chunker::DEFAULT_AVERAGE_SIZE;
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    average_size: Option<u32>,

    #[arg(short, long)]
    threads: Option<u32>,

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
    let average_size = cli.average_size.unwrap_or(DEFAULT_AVERAGE_SIZE);
    let threads = cli.threads.unwrap_or(num_cpus::get() as u32);

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    log::set_max_level(LevelFilter::Debug);

    let chunk_reader_writer = Arc::new(ChunkReaderWriter::new());
    let chunk_storage: Arc<Mutex<Box<dyn ChunkStorage + Send + Sync + 'static>>> =
        Arc::new(Mutex::new(Box::new(LocalChunkStorage::new())));

    match &cli.command {
        Some(Commands::Backup {
            input_path,
            output_path,
        }) => {
            let backup_config = Arc::new(BackupConfig::new(
                average_size,
                input_path,
                output_path,
                threads,
            ));

            let file_chunker = Arc::new(Mutex::new(FileChunker::new(
                backup_config.clone(),
                chunk_reader_writer.clone(),
                chunk_storage.clone(),
            )));
            let mut backup_service = BackupService::new(
                backup_config.clone(),
                file_chunker.clone(),
                chunk_reader_writer.clone(),
                chunk_storage.clone(),
            );
            backup_service.backup()?;
        }
        Some(Commands::Restore {
            input_path,
            output_path,
        }) => {
            let backup_config = Arc::new(BackupConfig::new(
                average_size,
                input_path,
                output_path,
                threads,
            ));

            let file_chunker = Arc::new(Mutex::new(FileChunker::new(
                backup_config.clone(),
                chunk_reader_writer.clone(),
                chunk_storage.clone(),
            )));
            let mut backup_service = BackupService::new(
                backup_config.clone(),
                file_chunker.clone(),
                chunk_reader_writer.clone(),
                chunk_storage.clone(),
            );
            backup_service.restore()?;
        }
        None => {}
    }

    Ok(())
}
