pub mod backup;

use core::str;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use log::{info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};

use backup::{backup::Backup, backup_config::BackupConfig, chunk_writer::ChunkWriter};

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
        Some(Commands::Backup {
            input_path,
            output_path,
        }) => {
            let backup_config = &BackupConfig::new(average_size, input_path, output_path);
            let chunk_writer = ChunkWriter::new(backup_config);
            let mut backup = Backup::new(backup_config, &chunk_writer);
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
