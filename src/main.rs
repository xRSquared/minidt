use anyhow::{anyhow, Result};
use minijinja::{context, path_loader, Environment};

use std::path::Path;
use std::path::PathBuf;

mod cli;
mod config;
mod constants;
mod styles;
mod utils;

fn main() -> Result<()> {
    let args = cli::parse_args();
    match args.command {
        | cli::Command::Init(init_args) => config::init_project(init_args),
        | cli::Command::Compile(compile_args) => compile_sql(compile_args),
    }
}

// TODO: allow for compiling from test.folder.file which would be relative the project root
fn compile_sql(compile_args: cli::CompileArgs) -> Result<()> {
    println!("Compiling SQL to remove Jinja");

    // Fetch project root directory
    let project_root = utils::find_project_config(constants::CONFIG_FILE_NAME)?;

    // Initialize MiniJinja environment with a loader
    let mut env = Environment::new();

    // Path to template folder from config
    let config = config::load_config(None)?;

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
        // Determine output directory path
        let output_dir = project_root.parent().unwrap().join(&config.outputs_folder);
        // If no output path is provided, construct the output directory based on the input file's parent directory
        let mut output_dir = output_dir.clone();
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
