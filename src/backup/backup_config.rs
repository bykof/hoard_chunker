use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupConfig {
    pub average_size: u32,
    pub input_path: String,
    pub output_path: String,
}

impl BackupConfig {
    pub fn new(average_size: u32, input_path: &Path, output_path: &Path) -> BackupConfig {
        BackupConfig {
            average_size,
            input_path: input_path.display().to_string(),
            output_path: output_path.display().to_string(),
        }
    }
    pub fn min_size(&self) -> u32 {
        return self.average_size / 4;
    }

    pub fn max_size(&self) -> u32 {
        return self.average_size / 4;
    }
}
