use anyhow::{anyhow, Result};
use clap::Parser;
use cli::InitArgs;
use minijinja::{context, path_loader, Environment};
use serde::{Deserialize, Serialize};

use std::io::prelude::*;
use std::path::Path;
use std::{fs::File, path::PathBuf};

mod cli;
mod constants;
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

fn find_project_config(config_filename: &str) -> Result<PathBuf> {
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

fn init_project(init_args: InitArgs) -> Result<()> {
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

fn create_defualt_config(config_file_path: &Path) -> Result<Config> {
    // If config file doesn't exist, create a default one
    let default_config = Config::default();
    save_config(&default_config, config_file_path)?;

    Ok(default_config)
}

fn load_config(config_file_path: Option<&Path>) -> Result<Config> {
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

// TODO: allow for compiling from test.folder.file which would be relative the project root
fn compile_sql(compile_args: cli::CompileArgs) -> Result<()> {
    println!("Compiling SQL to remove Jinja");

    // Fetch project root directory
    let project_root = find_project_config(constants::CONFIG_FILE_NAME)?;

    // Initialize MiniJinja environment with a loader
    let mut env = Environment::new();

    // Path to template folder from config
    let config = load_config(None)?;

    // Resolve the input file path
    let input_path = {
        let templates_abs_path = project_root
            .parent()
            .unwrap()
            .join(&config.templates_folder);

        // Convert the absolute path to a relative path relative to the templates folder
        if compile_args.file.is_absolute() {
            let absolute_input_path = compile_args.file.canonicalize()?;
            let relative_to_templates = absolute_input_path.strip_prefix(&templates_abs_path)?;

            PathBuf::from(relative_to_templates)
        } else {
            let relative_path = std::env::current_dir()?
                .join(&compile_args.file)
                .canonicalize()?;
            let relative_to_templates = relative_path.strip_prefix(&templates_abs_path)?;

            PathBuf::from(relative_to_templates)
        }
    };

    // Get the absolute path to the templates folder based on the project root
    let templates_abs_path =
        std::fs::canonicalize(project_root.parent().unwrap().join(config.templates_folder))
            .unwrap();
    println!("{:?}", templates_abs_path);

    // Set the loader with the absolute path to the templates folder
    env.set_loader(path_loader(templates_abs_path));

    println!("Loading templates from {:?}", input_path);
    // Compile SQL template

    let tmpl = env.get_template(input_path.to_str().unwrap())?;

    let compiled_sql = tmpl.render(context! {})?;

    // Determine output file path
    let output_path = if let Some(output) = compile_args.output {
        output
    } else {
        // If no output path is provided, construct the output directory based on the input file's parent directory
        let mut output_dir = PathBuf::from(&config.outputs_folder);
        if let Some(parent_dir) = input_path.parent() {
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
