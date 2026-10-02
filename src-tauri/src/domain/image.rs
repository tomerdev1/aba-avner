use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub path: PathBuf,
    pub hash: u64,
    pub size_bytes: u64,
}
