use anyhow::{anyhow, Result};
use clap::Parser;
use cli::InitArgs;
use minijinja::{context, path_loader, Environment};
use serde::{Deserialize, Serialize};

use std::io::prelude::*;
use std::path::Path;
use std::{fs::File, path::PathBuf};

mod cli;
mod styles;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    macros_folder: String,
    templates_folder: String,
    outputs_folder: String,
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

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    match args.commands {
        | cli::Commands::Init(init_args) => init_project(init_args),
        | cli::Commands::Compile(compile_args) => compile_sql(compile_args),
    }
}

fn init_project(init_args: InitArgs) -> Result<()> {
    let default_config_path = Path::new(".miniDT.toml");
    println!("Initializing a new project");

    let config = if let Some(config_path) = init_args.config_file {
        load_config(&config_path)?
    } else {
        load_config(default_config_path)?
    };

    create_directory(&config.macros_folder);
    create_directory(&config.templates_folder);
    create_directory(&config.outputs_folder);

    println!("Initialized a new project");
    Ok(())
}

fn load_config(config_file_path: &Path) -> Result<Config> {
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

fn save_config(config: &Config, config_file_path: &Path) -> Result<()> {
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

fn compile_sql(compile_args: cli::CompileArgs) -> Result<()> {
    println!("Compiling SQL to remove Jinja");

    // Initialize MiniJinja environment with a loader
    let mut env = Environment::new();

    // path to template folder from config
    // TODO: make this work from any directory nested within the project
    let config = load_config(Path::new(".miniDT.toml")).unwrap();

    env.set_loader(path_loader(config.templates_folder));

    // Compile SQL template
    let tmpl = env.get_template(compile_args.file.to_str().unwrap())?;
    let compiled_sql = tmpl.render(context! {})?;

    // Determine output directory path
    // Determine output file path
    let output_path = if let Some(output) = compile_args.output {
        output
    } else {
        // If no output path is provided, construct the output directory based on the input file's parent directory
        let mut output_dir = PathBuf::from(&config.outputs_folder);
        if let Some(parent_dir) = compile_args.file.parent() {
            output_dir.push(parent_dir);
        }

        // Create the output directory if it doesn't exist
        std::fs::create_dir_all(&output_dir)?;

        // NOTE: remove ".jinja" from the stem and keeping the extension ".sql"
        let mut output_filename = compile_args
            .file
            .file_stem()
            .ok_or_else(|| anyhow!("Invalid file name"))?
            .to_owned();

        if let Some(filename) = output_filename.to_str() {
            if let Some(idx) = filename.find(".jinja.") {
                output_filename = filename[..idx].to_string().into();
            }
        }

        output_filename = Path::new(&output_filename)
            .with_extension("sql")
            .file_name()
            .ok_or_else(|| anyhow!("Invalid file name"))?
            .to_owned();

        output_dir.push(output_filename);
        output_dir
    };

    // Write compiled SQL to output file
    std::fs::write(&output_path, compiled_sql)?;

    println!("Compiled SQL saved to {:?}", output_path);

    println!("{}", tmpl.render(context! {}).unwrap());
    Ok(())
}
