pub mod backup;

use core::str;
use std::{collections::HashMap, fs::File, path::PathBuf};

use clap::{Parser, Subcommand};

use fastcdc::v2020::*;
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use walkdir::WalkDir;

use backup::{
    backup::{Backup, BackupConfig},
    backup_metadata::BackupMetadata,
    chunk_table::Chunk,
    file_metadata::{FileChunk, FileMetadata},
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
    Scan {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    // 16KB
    let average_size = cli.average_size.unwrap_or(1024 * 16);
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
        Some(Commands::Scan {
            input_path,
            output_path,
        }) => {
            let backup = Backup::new(BackupConfig {
                average_size,
                input_path,
                output_path,
            });
            backup.backup()?;
        }
        Some(Commands::Restore {
            input_path,
            output_path,
        }) => {}
        None => {}
    }

    Ok(())
}
