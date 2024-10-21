use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Symlink {
    pub from: String,
    pub to: String,
}

impl Symlink {
    pub fn new<'a>(from: String, to: String) -> Symlink {
        Symlink { from, to }
    }
}
