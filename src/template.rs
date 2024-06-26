use anyhow::{anyhow, Result};
use minijinja::{context, path_loader, Environment};
use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    config::{self, Config},
    constants, utils,
};

// TODO: allow for compiling from test.folder.file which would be relative the project root
/// Compile the Jinja template to SQL.
pub fn compile_template(input_path: &Path, output_path: Option<&Path>) -> Result<()> {
    println!("Compiling SQL to remove Jinja");

    // Initialize MiniJinja environment with a loader
    let mut env = Environment::new();

    // Path to template folder from config
    let config = config::load_config(None)?;

    let project_root = utils::find_project_config(constants::CONFIG_FILE_NAME)?;
    let templates_abs_path = std::fs::canonicalize(
        project_root
            .parent()
            .unwrap()
            .join(&config.templates_folder),
    )
    .unwrap();

    let input_path = resolve_input_path(input_path, &templates_abs_path)?;

    // Set the loader with the absolute path to the templates folder
    env.set_loader(path_loader(templates_abs_path));

    // Load and render the template
    let tmpl = env.get_template(
        input_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid Template"))?,
    )?;
    let compiled_sql = tmpl.render(context! {})?;

    let output_path = resolve_output_path(output_path, &input_path, &config)?;

    write_output_file(&output_path, &compiled_sql)?;
    println!("Compiled SQL saved to {:?}", output_path);

    Ok(())
}

/// Resolve the input file path relative to the templates folder.
fn resolve_input_path(input_path: &Path, templates_abs_path: &PathBuf) -> Result<PathBuf> {
    let mut resolved_path = input_path.to_owned();

    if resolved_path.is_dir() {
        let config = config::load_config(None)?;
        let root_template_path = input_path.join(&config.root_template);

        if root_template_path.exists() {
            resolved_path = root_template_path;
        } else {
            return Err(
                anyhow!("\x1b[1;31mNo valid input file found.\x1b[0m \n Expected file '{}' in the directory '{}'",
                config.root_template, input_path.display())
            );
        }
    }

    if resolved_path.is_absolute() {
        let absolute_input_path = resolved_path.canonicalize()?;
        let relative_to_templates = absolute_input_path.strip_prefix(templates_abs_path)?;
        Ok(PathBuf::from(relative_to_templates))
    } else {
        let relative_path = std::env::current_dir()?
            .join(resolved_path)
            .canonicalize()?;
        let relative_to_templates = relative_path.strip_prefix(templates_abs_path)?;
        Ok(PathBuf::from(relative_to_templates))
    }
}

/// Resolve the output file path.
fn resolve_output_path(
    output_path: Option<&Path>,
    input_path: &Path,
    config: &Config,
) -> Result<PathBuf> {
    if let Some(output) = output_path {
        Ok(output.to_path_buf())
    } else {
        let project_root = utils::find_project_config(constants::CONFIG_FILE_NAME)?;
        let output_dir = project_root.parent().unwrap().join(&config.outputs_folder);
        let mut output_dir = output_dir.clone();
        if let Some(parent_dir) = input_path.parent() {
            output_dir.push(parent_dir);
        }
        let output_filename = input_path
            .file_stem()
            .ok_or_else(|| anyhow!("Invalid file name"))?
            .to_string_lossy()
            .replace(".jinja", "");
        let output_path = output_dir.join(output_filename).with_extension("sql");

        Ok(output_path)
    }
}

/// Write the compiled SQL output to a file.
fn write_output_file(output_path: &Path, compiled_sql: &str) -> Result<()> {
    println!("Writing compiled SQL to {:?}", output_path);

    // Create the output directory if it doesn't exist
    std::fs::create_dir_all(output_path.parent().expect("To create a directory"))?;

    let mut file = std::fs::File::create(output_path).expect("To create a file");
    file.write_all(compiled_sql.as_bytes())?;
    Ok(())
}
