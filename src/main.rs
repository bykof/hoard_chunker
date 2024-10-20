use core::str;
use std::{collections::HashMap, fs::File, path::PathBuf};

use clap::{Parser, Subcommand};

use fastcdc::v2020::*;
use hoard_chunker::{store_chunk, BackupMetadata, Chunk, FileChunk, FileMetadata};
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use walkdir::WalkDir;

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
        path: PathBuf,

        #[arg(short, long)]
        output_path: PathBuf,
    },
}

fn walk_path_and_scan(
    average_size: u32,
    path: &PathBuf,
    output_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let old_backup_metadata_result = BackupMetadata::deserialize(output_path);
    let mut backup_metadata = BackupMetadata::new();

    for result in WalkDir::new(path) {
        let entry = result.unwrap();
        let entry_path = entry.path().to_str().unwrap();
        let string_path = String::from(entry_path);

        info!("{}", entry.path().display());

        if entry.path().is_dir() {
            continue;
        }

        backup_metadata.file_metadatas.insert(
            string_path.clone(),
            FileMetadata {
                root_path: String::from(entry_path),
                chunks: HashMap::new(),
            },
        );

        let file = File::open(&string_path).expect("cannot open file!");
        let min_size = average_size / 4;
        let max_size = average_size * 4;
        let chunker = StreamCDC::new(file, min_size, average_size, max_size);

        for result in chunker {
            let chunk_data: ChunkData = result.expect("failed to read chunk");

            if !backup_metadata
                .chunk_table
                .chunk_map
                .contains_key(&chunk_data.hash)
            {
                store_chunk(&chunk_data, output_path);
                backup_metadata.chunk_table.chunk_map.insert(
                    chunk_data.hash.clone(),
                    Chunk {
                        hash: chunk_data.hash,
                        length: chunk_data.length,
                    },
                );
            }

            backup_metadata
                .file_metadatas
                .get_mut(&string_path)
                .expect("value must be there")
                .chunks
                .insert(
                    chunk_data.hash,
                    FileChunk {
                        hash: chunk_data.hash,
                        offset: chunk_data.offset.clone(),
                        length: chunk_data.length,
                    },
                );
        }
    }

    if old_backup_metadata_result.is_ok() {
        for (file_path, file_metadata) in &backup_metadata.file_metadatas {
            let old_backup_metadata = old_backup_metadata_result.as_ref().unwrap();

            if let Some(old_file_metadata) = old_backup_metadata.file_metadatas.get(file_path) {
                if file_metadata.fingerprint() != old_file_metadata.fingerprint() {
                    info!("Files are not identical");

                    let chunks: Vec<&u64> = file_metadata
                        .chunks
                        .keys()
                        .filter(|key| !old_file_metadata.chunks.contains_key(key))
                        .collect();

                    info!("New chunks: {:?}", chunks);
                } else {
                    info!("File are identical");
                }
            }
        }
    } else {
        error!("{:?}", old_backup_metadata_result.err())
    }

    backup_metadata.serialize(output_path);

    let sum = backup_metadata
        .chunk_table
        .chunk_map
        .values()
        .map(|value| value.length)
        .reduce(|a, b| a + b)
        .unwrap();

    info!("Stored: {} MB", sum / 1024 / 1024);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    // let size = &cli.size.unwrap_or(1024 * 128);
    let size = cli.average_size.unwrap_or(512);

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    log::set_max_level(LevelFilter::Debug);

    match &cli.command {
        Some(Commands::Scan { path, output_path }) => {
            walk_path_and_scan(size, path, output_path)?;
        }
        None => {}
    }

    Ok(())
}
