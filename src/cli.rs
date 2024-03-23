use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(name="miniDT")]
#[command(about,long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands:Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands{
    /// Initialize a new project
    Init(InitArgs),
}



#[derive(Args, Debug)]
pub struct InitArgs {
    pub config_file: Option<PathBuf>,
}
