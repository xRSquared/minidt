use crate::cli::InitArgs;
use crate::constants;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;

///NOTE:  utils ////////////////////
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

fn is_project_initialized(config_filename: &str) -> Option<PathBuf> {
    match find_project_config(config_filename) {
        | Ok(config_path) => Some(config_path),
        | Err(_) => None,
    }
}

fn create_directory(name: &str) {
    // Errors aren't really errors. If  directory already exists, just ignore it
    std::fs::create_dir(name).unwrap_or(())
}

//////NOTE: end of utils ////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub macros_folder: String,
    pub templates_folder: String,
    pub outputs_folder: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            macros_folder: "macros".to_string(),
            templates_folder: "models".to_string(),
            outputs_folder: "compiled".to_string(),
        }
    }
}

pub fn create_defualt_config(config_file_path: &Path) -> Result<Config> {
    // If config file doesn't exist, create a default one
    let default_config = Config::default();
    save_config(&default_config, config_file_path)?;

    Ok(default_config)
}

pub fn load_config(config_file_path: Option<&Path>) -> Result<Config> {
    let file_path = match config_file_path {
        | Some(path) => path.to_owned(),
        | None => find_project_config(constants::CONFIG_FILE_NAME)?,
    };

    let mut file = File::open(file_path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}

pub fn save_config(config: &Config, config_file_path: &Path) -> Result<()> {
    let toml = toml::to_string(config)?;

    let mut file = File::create(config_file_path)
        .map_err(|e| anyhow!("Failed to create config file: {}", e))?;

    file.write_all(toml.as_bytes())
        .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

    Ok(())
}

pub fn init_project(init_args: InitArgs) -> Result<()> {
    println!("Initializing a new project");

    let config = if let Some(config_path) = init_args.config_file {
        load_config(Some(&config_path))?
    } else {
        // NOTE: Check if the project is already initialized and error with warning
        if let Some(config_path) = is_project_initialized(constants::CONFIG_FILE_NAME) {
            return Err(anyhow!(
            format!("\u{1b}[1;33mProject already initialized.\u{1b}[0m\n Configuration file found at:  {:?}.\n Skipping initialization.", config_path),
        ));
        }

        // Can do this better, but for now good enough...
        match load_config(None) {
            | Ok(config) => config,
            | Err(_) => create_defualt_config(Path::new(constants::CONFIG_FILE_NAME))?,
        }
    };

    create_directory(&config.macros_folder);
    create_directory(&config.templates_folder);
    create_directory(&config.outputs_folder);

    println!("Initialized a new project");
    Ok(())
}
