use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn find_project_config(config_filename: &str) -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir().expect("to get current directory");

    loop {
        let config_file_path = current_dir.join(config_filename);
        if config_file_path.is_file() {
            return Ok(config_file_path);
        }

        if !current_dir.pop() {
            // We have reached the root directory
            break;
        }
    }
    Err(anyhow!("\u{1b}[1;31mNo Configuration file found.\u{1b}[0m"))
}
