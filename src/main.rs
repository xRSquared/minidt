use anyhow::{anyhow, Result};
use clap::Parser;
use cli::InitArgs;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

mod cli;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    macros_folder: String,
    models_folder: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            macros_folder: "macros".to_string(),
            models_folder: "models".to_string(),
        }
    }
}

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    match args.commands {
        | cli::Commands::Init(init_args) => init_project(init_args),
    }
}

fn init_project(init_args: InitArgs) -> Result<()> {
    let default_config_path = PathBuf::from(".miniDT.toml");
    println!("Initializing a new project");

    let config = if let Some(config_path) = init_args.config_file {
        load_config(&config_path)?
    } else {
        load_config(&default_config_path)?
    };

    create_directory(&config.macros_folder);
    create_directory(&config.models_folder);

    println!("Initialized a new project");
    Ok(())
}

fn load_config(config_file_path: &PathBuf) -> Result<Config> {
    if std::fs::metadata(config_file_path).is_err() {
        // If config file doesn't exist, create a default one
        let default_config = Config::default();
        save_config(&default_config, config_file_path)?;

        return Ok(default_config);
    }

    let mut file = File::open(config_file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let config: Config = toml::from_str(&contents)?;

    Ok(config)
}

fn save_config(config: &Config, config_file_path: &PathBuf) -> Result<()> {
    let toml = toml::to_string(config)?;

    let mut file = File::create(config_file_path)
        .map_err(|e| anyhow!("Failed to create config file: {}", e))?;

    file.write_all(toml.as_bytes())
        .map_err(|e| anyhow!("Failed to write config file: {}", e))?;

    Ok(())
}

fn create_directory(name: &str) {
    // Errors aren't really errors. If  directory already exists, just ignore it
    std::fs::create_dir(name).unwrap_or(())
}
