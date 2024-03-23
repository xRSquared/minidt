use crate::styles::get_styles;
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(name = "miniDT")]
#[command(about,long_about=None)]
#[command(styles=get_styles())]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new project
    Init(InitArgs),
    /// Compile SQL to remove jinja
    Compile(CompileArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {
    pub config_file: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct CompileArgs {
    /// Path to the sql file to compile
    pub file: PathBuf,

    /// Path to store the output file [default is to remove "jinja" from the file name]
    pub output: Option<PathBuf>,

    /// Output type
    #[clap(short = 't', long, default_value_t, value_enum)]
    pub output_type: OutputType,
}

#[derive(clap::ValueEnum, Debug, Clone, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputType {
    #[default]
    /// Create a view
    View,
    /// Create a table
    Table,
    /// Create a temporary table
    TempTable,
}
