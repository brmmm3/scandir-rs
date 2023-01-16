use std::io::Error;

use tempfile::TempDir;

pub fn setup() -> TempDir {
    let base_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let _ = std::env::set_current_dir(&base_dir);
    TempDir::new().unwrap()
}

#[allow(dead_code)]
pub fn cleanup(temp_dir: TempDir) -> Result<(), Error> {
    temp_dir.close()
}
